import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';

// Maintenance tool only. The application and normal builds do not need Node or sharp.
const require = createRequire(import.meta.url);
let sharp;
try {
  sharp = require(process.argv[2] || 'sharp');
} catch (error) {
  throw new Error(
    'Install sharp for icon maintenance, then run: node scripts/generate-icons.mjs /path/to/node_modules/sharp',
    { cause: error },
  );
}
const directory = new URL('../assets/icons/', import.meta.url);
const source = await readFile(new URL('keyscribe.svg', directory));
const images = new Map();
for (const size of [16, 24, 32, 48, 64, 128, 256, 512, 1024]) {
  images.set(size, await sharp(source, { density: 192 })
    .resize(size, size)
    .png({ compressionLevel: 9 })
    .toBuffer());
}

const sizes = [16, 24, 32, 48, 64, 128, 256];
const directorySize = 6 + 16 * sizes.length;
const icoHeader = Buffer.alloc(directorySize);
icoHeader.writeUInt16LE(1, 2);
icoHeader.writeUInt16LE(sizes.length, 4);
let offset = directorySize;
for (const [index, size] of sizes.entries()) {
  const entry = 6 + 16 * index;
  const png = images.get(size);
  icoHeader[entry] = size === 256 ? 0 : size;
  icoHeader[entry + 1] = size === 256 ? 0 : size;
  icoHeader.writeUInt16LE(1, entry + 4);
  icoHeader.writeUInt16LE(32, entry + 6);
  icoHeader.writeUInt32LE(png.length, entry + 8);
  icoHeader.writeUInt32LE(offset, entry + 12);
  offset += png.length;
}
const ico = Buffer.concat([icoHeader, ...sizes.map(size => images.get(size))]);

// PNG representations for standard and Retina sizes on the supported macOS versions.
const representations = [
  ['icp4', 16], ['icp5', 32], ['icp6', 64], ['ic07', 128],
  ['ic08', 256], ['ic09', 512], ['ic10', 1024],
  ['ic11', 32], ['ic12', 64], ['ic13', 256], ['ic14', 512],
];
const chunks = representations.map(([type, size]) => {
  const png = images.get(size);
  const header = Buffer.alloc(8);
  header.write(type, 0, 4, 'ascii');
  header.writeUInt32BE(png.length + 8, 4);
  return Buffer.concat([header, png]);
});
const icnsHeader = Buffer.alloc(8);
icnsHeader.write('icns', 0, 4, 'ascii');
icnsHeader.writeUInt32BE(8 + chunks.reduce((sum, chunk) => sum + chunk.length, 0), 4);

await mkdir(directory, { recursive: true });
for (const [name, data] of [
  ['keyscribe-128.png', images.get(128)],
  ['keyscribe.ico', ico],
  ['keyscribe.icns', Buffer.concat([icnsHeader, ...chunks])],
]) {
  await writeFile(new URL(name, directory), data);
  console.log(`${fileURLToPath(new URL(name, directory))}: ${data.length} bytes`);
}
