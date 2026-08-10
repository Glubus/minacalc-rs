const SVG_NS = 'http://www.w3.org/2000/svg';

export const SKILLSETS = [
  ['stream', 'Stream'],
  ['jumpstream', 'Jumpstream'],
  ['handstream', 'Handstream'],
  ['stamina', 'Stamina'],
  ['jackspeed', 'Jackspeed'],
  ['chordjack', 'Chordjack'],
  ['technical', 'Technical'],
];

export function renderRadar(container, scores) {
  const width = 430;
  const height = 350;
  const centerX = width / 2;
  const centerY = 170;
  const radius = 120;
  const maximum = Math.max(5, Math.ceil(Math.max(...SKILLSETS.map(([key]) => scores[key])) / 5) * 5);
  const svg = svgElement('svg', { viewBox: `0 0 ${width} ${height}`, role: 'img', 'aria-label': 'Skillset radar chart' });

  for (let ring = 1; ring <= 4; ring += 1) {
    const points = polygonPoints(SKILLSETS.length, centerX, centerY, radius * ring / 4);
    svg.append(svgElement('polygon', { points, class: 'radar-grid' }));
  }

  SKILLSETS.forEach(([, label], index) => {
    const angle = axisAngle(index, SKILLSETS.length);
    const end = polarPoint(centerX, centerY, radius, angle);
    const text = polarPoint(centerX, centerY, radius + 25, angle);
    svg.append(svgElement('line', { x1: centerX, y1: centerY, x2: end.x, y2: end.y, class: 'radar-axis' }));
    svg.append(svgText(label, text.x, text.y, labelAnchor(Math.cos(angle)), 'radar-label'));
  });

  const values = SKILLSETS.map(([key]) => scores[key]);
  const shape = values.map((value, index) => {
    const point = polarPoint(centerX, centerY, radius * value / maximum, axisAngle(index, values.length));
    return `${point.x},${point.y}`;
  }).join(' ');
  svg.append(svgElement('polygon', { points: shape, class: 'radar-shape' }));

  values.forEach((value, index) => {
    const point = polarPoint(centerX, centerY, radius * value / maximum, axisAngle(index, values.length));
    svg.append(svgElement('circle', { cx: point.x, cy: point.y, r: 3.6, class: 'radar-dot' }));
  });

  container.replaceChildren(svg);
}

export function renderTrend(container, results, selectedIndex, onSelect) {
  const width = 680;
  const height = 335;
  const padding = { top: 35, right: 28, bottom: 45, left: 52 };
  const chartWidth = width - padding.left - padding.right;
  const chartHeight = height - padding.top - padding.bottom;
  const overalls = results.map((result) => result.scores.overall);
  const rates = results.map((result) => result.rate);
  const minRate = Math.min(...rates);
  const maxRate = Math.max(...rates);
  const rawMin = Math.min(...overalls);
  const rawMax = Math.max(...overalls);
  const yPadding = Math.max(1, (rawMax - rawMin) * .16);
  const minY = Math.max(0, rawMin - yPadding);
  const maxY = rawMax + yPadding;
  const baseline = closestToOne(results).scores.overall;
  const svg = svgElement('svg', { viewBox: `0 0 ${width} ${height}`, role: 'img', 'aria-label': 'Overall rating progression by rate' });

  const definitions = svgElement('defs');
  const gradient = svgElement('linearGradient', { id: 'trend-gradient', x1: '0', y1: '0', x2: '0', y2: '1' });
  gradient.append(svgElement('stop', { offset: '0%', 'stop-color': '#8d79ff', 'stop-opacity': '.32' }));
  gradient.append(svgElement('stop', { offset: '100%', 'stop-color': '#8d79ff', 'stop-opacity': '0' }));
  definitions.append(gradient);
  svg.append(definitions);

  for (let index = 0; index <= 4; index += 1) {
    const ratio = index / 4;
    const y = padding.top + chartHeight * ratio;
    const value = maxY - (maxY - minY) * ratio;
    svg.append(svgElement('line', { x1: padding.left, y1: y, x2: width - padding.right, y2: y, class: 'trend-grid' }));
    svg.append(svgText(value.toFixed(1), padding.left - 10, y + 4, 'end', 'trend-label'));
  }

  const points = results.map((result, index) => ({
    x: scale(result.rate, minRate, maxRate, padding.left, padding.left + chartWidth, index, results.length),
    y: scale(result.scores.overall, minY, maxY, padding.top + chartHeight, padding.top),
  }));
  const pointString = points.map((point) => `${point.x},${point.y}`).join(' ');
  const areaPoints = `${padding.left},${padding.top + chartHeight} ${pointString} ${padding.left + chartWidth},${padding.top + chartHeight}`;
  svg.append(svgElement('polygon', { points: areaPoints, class: 'trend-area' }));
  svg.append(svgElement('polyline', { points: pointString, class: 'trend-line' }));

  points.forEach((point, index) => {
    const result = results[index];
    const delta = result.scores.overall - baseline;
    const pointNode = svgElement('circle', {
      cx: point.x,
      cy: point.y,
      r: index === selectedIndex ? 7 : 5,
      class: `trend-point${index === selectedIndex ? ' selected' : ''}`,
      tabindex: '0',
      role: 'button',
      'aria-label': `${result.rate.toFixed(2)}x, ${result.scores.overall.toFixed(2)} overall, ${formatDelta(delta)} versus 1.00x`,
    });
    pointNode.append(svgElement('title', {}, `${result.rate.toFixed(2)}x · ${result.scores.overall.toFixed(2)} · ${formatDelta(delta)} vs 1.00x`));
    pointNode.addEventListener('click', () => onSelect(index));
    pointNode.addEventListener('keydown', (event) => {
      if (event.key === 'Enter' || event.key === ' ') onSelect(index);
    });
    svg.append(pointNode);

    svg.append(svgText(`${result.rate.toFixed(2)}x`, point.x, height - 19, 'middle', 'trend-label'));
    if (results.length <= 10 && Math.abs(delta) >= .005) {
      svg.append(svgText(formatDelta(delta), point.x, point.y - 13, 'middle', 'trend-delta'));
    }
  });

  container.replaceChildren(svg);
}

export function closestToOne(results) {
  return results.reduce((closest, result) => (
    Math.abs(result.rate - 1) < Math.abs(closest.rate - 1) ? result : closest
  ));
}

export function formatDelta(value) {
  if (Math.abs(value) < .005) return '±0.00';
  return `${value > 0 ? '+' : ''}${value.toFixed(2)}`;
}

function polygonPoints(count, centerX, centerY, radius) {
  return Array.from({ length: count }, (_, index) => {
    const point = polarPoint(centerX, centerY, radius, axisAngle(index, count));
    return `${point.x},${point.y}`;
  }).join(' ');
}

function axisAngle(index, count) {
  return -Math.PI / 2 + index * Math.PI * 2 / count;
}

function polarPoint(centerX, centerY, radius, angle) {
  return { x: centerX + Math.cos(angle) * radius, y: centerY + Math.sin(angle) * radius };
}

function labelAnchor(cosine) {
  if (cosine > .25) return 'start';
  if (cosine < -.25) return 'end';
  return 'middle';
}

function scale(value, min, max, outputMin, outputMax, index = 0, count = 1) {
  if (max === min) {
    return count === 1 ? (outputMin + outputMax) / 2 : outputMin + (outputMax - outputMin) * index / (count - 1);
  }
  return outputMin + (value - min) / (max - min) * (outputMax - outputMin);
}

function svgText(content, x, y, anchor, className) {
  const node = svgElement('text', { x, y, 'text-anchor': anchor, class: className });
  node.textContent = content;
  return node;
}

function svgElement(name, attributes = {}, text = null) {
  const node = document.createElementNS(SVG_NS, name);
  Object.entries(attributes).forEach(([key, value]) => node.setAttribute(key, String(value)));
  if (text !== null) node.textContent = text;
  return node;
}
