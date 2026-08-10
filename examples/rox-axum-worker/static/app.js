const form = document.querySelector('#rating-form');
const button = form.querySelector('button');
const status = document.querySelector('#status');
const results = document.querySelector('#results');
const summary = document.querySelector('#summary');
const ratings = document.querySelector('#ratings');
const skillsets = [
  'overall',
  'stream',
  'jumpstream',
  'handstream',
  'stamina',
  'jackspeed',
  'chordjack',
  'technical',
];

form.addEventListener('submit', async (event) => {
  event.preventDefault();
  if (!hasExactlyOneChartSource()) return;

  setLoading(true);
  try {
    const response = await fetch('/api/rate', {
      method: 'POST',
      body: new FormData(form),
    });
    const body = await response.json();
    if (!response.ok) throw new Error(body.error || `HTTP ${response.status}`);
    renderResponse(body);
  } catch (error) {
    showError(error instanceof Error ? error.message : String(error));
  } finally {
    setLoading(false);
  }
});

function hasExactlyOneChartSource() {
  const url = form.elements.osu_url.value.trim();
  const file = form.elements.chart.files[0];
  if (Boolean(url) === Boolean(file)) {
    showError('Provide either an osu! URL or a chart file, not both.');
    return false;
  }
  return true;
}

function setLoading(loading) {
  button.disabled = loading;
  if (loading) {
    status.className = '';
    status.textContent = 'Downloading, parsing and calculating…';
    results.hidden = true;
  }
}

function renderResponse(body) {
  const chartName = body.title || body.file_name || 'Untitled chart';
  summary.textContent = `${chartName} — ${body.key_count}K, ${body.row_count} rows from ${body.source_note_count} source notes, ${body.mode.toUpperCase()}`;
  ratings.replaceChildren(...body.results.map(createRatingRow));
  status.className = '';
  status.textContent = `Calculated ${body.results.length} rate${body.results.length === 1 ? '' : 's'}.`;
  results.hidden = false;
}

function createRatingRow(result) {
  const row = document.createElement('tr');
  const values = [
    `${result.rate.toFixed(2)}x`,
    ...skillsets.map((name) => result.scores[name].toFixed(2)),
  ];
  row.replaceChildren(...values.map(createCell));
  return row;
}

function createCell(value) {
  const cell = document.createElement('td');
  cell.textContent = value;
  return cell;
}

function showError(message) {
  status.className = 'error';
  status.textContent = message;
  results.hidden = true;
}
