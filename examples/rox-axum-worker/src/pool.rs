use std::{fmt, io, num::NonZeroUsize, sync::mpsc, thread};

use async_channel::{Receiver, Sender};
use minacalc_rs::Calc;
use tokio::sync::oneshot;

use crate::{
    calculator,
    error::ApiError,
    models::{RatingRequest, RatingResponse},
};

const QUEUED_JOBS_PER_WORKER: usize = 2;

struct Job {
    request: RatingRequest,
    reply: oneshot::Sender<Result<RatingResponse, ApiError>>,
}

#[derive(Clone)]
pub(crate) struct CalculatorPool {
    jobs: Sender<Job>,
}

impl CalculatorPool {
    pub(crate) fn for_available_parallelism() -> Result<Self, CalculatorPoolInitError> {
        let worker_count = thread::available_parallelism()
            .map(NonZeroUsize::get)
            .unwrap_or(1);
        Self::new(worker_count)
    }

    fn new(worker_count: usize) -> Result<Self, CalculatorPoolInitError> {
        let queue_capacity = worker_count.saturating_mul(QUEUED_JOBS_PER_WORKER);
        let (jobs, receiver) = async_channel::bounded(queue_capacity);
        let (ready, readiness) = mpsc::channel();

        for worker_index in 0..worker_count {
            let receiver = receiver.clone();
            let ready = ready.clone();
            thread::Builder::new()
                .name(format!("minacalc-{worker_index}"))
                .spawn(move || run_worker(worker_index, receiver, ready))
                .map_err(|source| CalculatorPoolInitError::Spawn {
                    worker_index,
                    source,
                })?;
        }
        drop(ready);

        for _ in 0..worker_count {
            match readiness.recv() {
                Ok((_, Ok(()))) => {}
                Ok((worker_index, Err(source))) => {
                    return Err(CalculatorPoolInitError::Calculator {
                        worker_index,
                        source,
                    });
                }
                Err(_) => return Err(CalculatorPoolInitError::StartupChannel),
            }
        }

        Ok(Self { jobs })
    }

    pub(crate) async fn rate(&self, request: RatingRequest) -> Result<RatingResponse, ApiError> {
        let (reply, response) = oneshot::channel();
        self.jobs
            .send(Job { request, reply })
            .await
            .map_err(|_| ApiError::internal("calculator pool is unavailable"))?;
        response
            .await
            .map_err(|_| ApiError::internal("calculator worker stopped unexpectedly"))?
    }
}

fn run_worker(
    worker_index: usize,
    jobs: Receiver<Job>,
    ready: mpsc::Sender<(usize, Result<(), minacalc_rs::Error>)>,
) {
    let mut calc = match Calc::new() {
        Ok(calc) => calc,
        Err(error) => {
            let _ = ready.send((worker_index, Err(error)));
            return;
        }
    };
    if ready.send((worker_index, Ok(()))).is_err() {
        return;
    }

    while let Ok(job) = jobs.recv_blocking() {
        if !job.reply.is_closed() {
            let result = calculator::rate(&mut calc, job.request);
            let _ = job.reply.send(result);
        }
    }
}

/// Failure to start the calculator worker pool.
#[derive(Debug)]
pub enum CalculatorPoolInitError {
    Spawn {
        worker_index: usize,
        source: io::Error,
    },
    Calculator {
        worker_index: usize,
        source: minacalc_rs::Error,
    },
    StartupChannel,
}

impl fmt::Display for CalculatorPoolInitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spawn {
                worker_index,
                source,
            } => write!(
                formatter,
                "could not start calculator worker {worker_index}: {source}"
            ),
            Self::Calculator {
                worker_index,
                source,
            } => write!(
                formatter,
                "could not initialize calculator worker {worker_index}: {source}"
            ),
            Self::StartupChannel => {
                formatter.write_str("calculator worker stopped during initialization")
            }
        }
    }
}

impl std::error::Error for CalculatorPoolInitError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Spawn { source, .. } => Some(source),
            Self::Calculator { source, .. } => Some(source),
            Self::StartupChannel => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use minacalc_rs::CalcConfig;

    use super::*;
    use crate::models::{ChartPayload, RatingMode};

    fn request(config: CalcConfig) -> RatingRequest {
        RatingRequest {
            chart: ChartPayload {
                bytes: include_bytes!("../../../crates/minacalc-sys/assets/4K.osu").to_vec(),
                file_name: Some("4K.osu".to_owned()),
            },
            rates: vec![1.0],
            mode: RatingMode::Msd,
            config,
        }
    }

    #[tokio::test]
    async fn one_worker_handles_and_reconfigures_multiple_jobs() {
        let pool = CalculatorPool::new(1).unwrap();
        let first = pool.rate(request(CalcConfig::default())).await.unwrap();
        let config = CalcConfig {
            ssr_goal_cap: 1.0,
            ..CalcConfig::default()
        };
        let second = pool.rate(request(config)).await.unwrap();

        assert_eq!(first.results.len(), 1);
        assert_eq!(second.results.len(), 1);
    }
}
