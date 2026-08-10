import { SKILLSETS, closestToOne, formatDelta, renderRadar, renderTrend } from './charts.js';

const FILTER_DEFAULTS = {
  rates: '0.80, 0.90, 1.00, 1.10, 1.20, 1.30, 1.40, 1.50',
  mode: 'msd',
  score_goal: '0.93',
  ssr_goal_cap: '0.965',
  low_acc_cutoff: '0.90',
  ssr_rating_cap: '40',
  grind_scaling: 'true',
  scaler_stream: '1.00',
  scaler_jumpstream: '1.00',
  scaler_handstream: '1.00',
  scaler_stamina: '1.00',
  scaler_jackspeed: '1.00',
  scaler_chordjack: '1.00',
  scaler_technical: '1.00',
};

const form = document.querySelector('#rating-form');
const rateButton = document.querySelector('#rate-button');
const filtersDialog = document.querySelector('#filters-dialog');
const filterCount = document.querySelector('#filter-count');
const status = document.querySelector('#status');
const loading = document.querySelector('#loading');
const resultsSection = document.querySelector('#results');
const radarRate = document.querySelector('#radar-rate');
const tableBody = document.querySelector('#ratings-table');
let currentResponse = null;
let selectedIndex = 0;

document.querySelector('#open-filters').addEventListener('click', () => filtersDialog.showModal());
document.querySelector('#close-filters').addEventListener('click', () => filtersDialog.close());
document.querySelector('#apply-filters').addEventListener('click', () => filtersDialog.close());
document.querySelector('#reset-filters').addEventListener('click', resetFilters);
filtersDialog.addEventListener('click', closeOnBackdrop);
radarRate.addEventListener('change', () => selectRate(Number(radarRate.value)));

Object.keys(FILTER_DEFAULTS).forEach((name) => {
  form.elements[name].addEventListener('input', updateFilterCount);
});

form.addEventListener('submit', async (event) => {
  event.preventDefault();
  if (!form.reportValidity()) return;

  setLoading(true);
  try {
    const response = await fetch('/api/rate', { method: 'POST', body: new FormData(form) });
    const body = await response.json();
    if (!response.ok) throw new Error(body.error || `HTTP ${response.status}`);
    renderResponse(body);
  } catch (error) {
    showError(error instanceof Error ? error.message : String(error));
  } finally {
    setLoading(false);
  }
});

function renderResponse(body) {
  currentResponse = body;
  selectedIndex = body.results.indexOf(closestToOne(body.results));
  renderHero(body);
  renderMetadata(body);
  renderRateOptions(body.results);
  renderTable(body.results);
  selectRate(selectedIndex);

  status.className = 'status';
  status.textContent = `Calculated ${body.results.length} rates · ${body.title || body.file_name}`;
  resultsSection.hidden = false;
  resultsSection.scrollIntoView({ behavior: 'smooth', block: 'start' });
}

function renderHero(body) {
  setCover(document.querySelector('#chart-hero'), body.cover_url);
  setText('#hero-keys', `${body.key_count}K`);
  setText('#hero-mode', body.mode.toUpperCase());
  setText('#hero-duration', formatDuration(body.duration_seconds));
  setText('#hero-artist', body.artist || 'Unknown artist');
  setText('#hero-title', body.title || body.file_name || 'Untitled chart');
  setText('#hero-difficulty', body.difficulty || 'Unnamed difficulty');
  setText('#hero-mapper', body.creator ? `Mapped by ${body.creator}` : 'Mapper unknown');
}

function setCover(hero, coverUrl) {
  hero.classList.remove('has-cover');
  hero.style.backgroundImage = '';
  if (!coverUrl?.startsWith('https://assets.ppy.sh/')) return;

  const image = new Image();
  image.onload = () => {
    hero.style.backgroundImage = `url("${coverUrl}")`;
    hero.classList.add('has-cover');
  };
  image.src = coverUrl;
}

function renderMetadata(body) {
  setText('#meta-rows', body.row_count.toLocaleString());
  setText('#meta-notes', body.source_note_count.toLocaleString());
  setText('#meta-goal', `${(body.score_goal * 100).toFixed(1)}%`);
}

function renderRateOptions(rateResults) {
  radarRate.replaceChildren(...rateResults.map((result, index) => {
    const option = document.createElement('option');
    option.value = String(index);
    option.textContent = `${result.rate.toFixed(2)}x`;
    return option;
  }));
}

function renderTable(rateResults) {
  const baseline = closestToOne(rateResults).scores.overall;
  tableBody.replaceChildren(...rateResults.map((result, index) => {
    const row = document.createElement('tr');
    const delta = result.scores.overall - baseline;
    const cells = [
      `${result.rate.toFixed(2)}x`,
      result.scores.overall.toFixed(2),
      formatDelta(delta),
      ...SKILLSETS.map(([key]) => result.scores[key].toFixed(2)),
    ];
    row.replaceChildren(...cells.map((value, cellIndex) => createTableCell(value, cellIndex, delta)));
    row.addEventListener('click', () => selectRate(index));
    return row;
  }));
}

function createTableCell(value, index, delta) {
  const cell = document.createElement('td');
  const content = index === 1 ? document.createElement('strong') : document.createElement('span');
  content.textContent = value;
  if (index === 2) content.className = deltaClass(delta);
  cell.append(content);
  return cell;
}

function selectRate(index) {
  if (!currentResponse || !currentResponse.results[index]) return;
  selectedIndex = index;
  const selected = currentResponse.results[index];
  const baseline = closestToOne(currentResponse.results).scores.overall;
  const delta = selected.scores.overall - baseline;

  radarRate.value = String(index);
  setText('#selected-rate', `${selected.rate.toFixed(2)}x`);
  setText('#selected-overall', selected.scores.overall.toFixed(2));
  setText('#selected-delta', formatDelta(delta));
  document.querySelector('#selected-delta').className = deltaClass(delta);
  renderRadar(document.querySelector('#radar-chart'), selected.scores);
  renderTrend(document.querySelector('#trend-chart'), currentResponse.results, index, selectRate);
  renderSkillsetValues(selected.scores);
  [...tableBody.rows].forEach((row, rowIndex) => row.classList.toggle('selected', rowIndex === index));
}

function renderSkillsetValues(scores) {
  const container = document.querySelector('#skillset-values');
  container.replaceChildren(...SKILLSETS.map(([key, label]) => {
    const item = document.createElement('div');
    item.className = 'skill-value';
    const name = document.createElement('span');
    const value = document.createElement('strong');
    name.textContent = label;
    value.textContent = scores[key].toFixed(2);
    item.append(name, value);
    return item;
  }));
}

function setLoading(isLoading) {
  rateButton.disabled = isLoading;
  loading.hidden = !isLoading;
  if (isLoading) {
    resultsSection.hidden = true;
    status.className = 'status';
    status.textContent = 'Downloading the map, parsing notes and calculating rates…';
  }
}

function showError(message) {
  status.className = 'status error';
  status.textContent = message;
  loading.hidden = true;
  resultsSection.hidden = true;
}

function resetFilters() {
  Object.entries(FILTER_DEFAULTS).forEach(([name, value]) => {
    form.elements[name].value = value;
  });
  updateFilterCount();
}

function updateFilterCount() {
  const modified = Object.entries(FILTER_DEFAULTS).filter(([name, value]) => (
    String(form.elements[name].value).trim() !== value
  )).length;
  filterCount.textContent = String(modified);
  filterCount.hidden = modified === 0;
}

function closeOnBackdrop(event) {
  if (event.target === filtersDialog) filtersDialog.close();
}

function deltaClass(delta) {
  if (delta > .005) return 'positive delta-positive';
  if (delta < -.005) return 'negative delta-negative';
  return '';
}

function formatDuration(seconds) {
  const minutes = Math.floor(seconds / 60);
  const remaining = Math.round(seconds % 60).toString().padStart(2, '0');
  return `${minutes}:${remaining}`;
}

function setText(selector, value) {
  document.querySelector(selector).textContent = value;
}
