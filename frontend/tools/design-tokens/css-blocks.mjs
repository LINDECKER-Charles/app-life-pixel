// Just enough CSS and SCSS reading to compare two token files: the body of a rule or mixin, its
// custom properties, and the channels of a colour. Neither file nests a rule inside a token block.

/** The text between the braces of the first block that follows `marker`, nested blocks included. */
export function blockAfter(text, marker) {
  const start = text.indexOf(marker);
  if (start < 0) throw new Error(`no block after ${marker}`);
  const open = text.indexOf('{', start);
  let depth = 0;
  for (let index = open; index < text.length; index++) {
    if (text[index] === '{') depth++;
    if (text[index] === '}' && --depth === 0) return text.slice(open + 1, index);
  }
  throw new Error(`unclosed block after ${marker}`);
}

/** The custom properties declared in a block, values lowercased with their spaces collapsed. */
export function customProperties(block) {
  const text = block.replace(/\/\*[\s\S]*?\*\//g, '').replace(/^\s*\/\/.*$/gm, '');
  const properties = new Map();
  for (const chunk of text.split(';')) {
    const match = /^\s*(--[\w-]+)\s*:\s*([\s\S]+?)\s*$/.exec(chunk);
    if (match) properties.set(match[1], match[2].replace(/\s+/g, ' ').toLowerCase());
  }
  return properties;
}

/** The red, green and blue channels of `#rrggbb`, `rgb(r g b / a)` or an Ionic `r, g, b` list. */
export function channels(value) {
  const hex = /^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/.exec(value);
  if (hex) return hex.slice(1).map((pair) => Number.parseInt(pair, 16));
  const numbers = /^(?:rgb\()?\s*(\d+)[\s,]+(\d+)[\s,]+(\d+)/.exec(value);
  if (numbers) return numbers.slice(1).map(Number);
  throw new Error(`not an sRGB colour: ${value}`);
}

/** The opacity of `rgb(r g b / a%)`, from 0 to 1. */
export function alpha(value) {
  const match = /\/\s*([\d.]+)%\s*\)$/.exec(value);
  if (!match) throw new Error(`no percentage alpha in ${value}`);
  return Number(match[1]) / 100;
}
