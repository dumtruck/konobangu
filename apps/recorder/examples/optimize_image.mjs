#!/usr/bin/env zx
import { glob } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { chunk } from 'es-toolkit/array';

const dataDir = path.join(import.meta.dirname, '../../../data')
/**
 * @type {string[]}
 */
const images = [];
for await (const image of glob('**/*.{jpg,jpeg,png,gif,svg}', {
    cwd: dataDir,
})) {
    images.push(image)
}

const cpus = os.cpus().length - 1;

const chunkSize = Math.ceil(images.length / cpus);
const chunks = chunk(images, chunkSize);

/**
 * @param {string[]} images
 */
async function convertImages(images) {
    for await (const image of images) {
        const imagePath = path.resolve(dataDir, image)
        const webp = imagePath.replace(path.extname(imagePath), '.webp')
        const avif = imagePath.replace(path.extname(imagePath), '.avif')
        console.log(`Converting ${imagePath} to ${webp}...`);
        await $`ffmpeg -i "${imagePath}" -c:v libwebp -lossless 1 "${webp}"`;
        console.log(`Converting ${imagePath} to ${avif}...`);
        await $`ffmpeg -i "${imagePath}" -c:v libaom-av1 -still-picture 1 -pix_fmt yuv420p10le -crf 0 -strict experimental "${avif}"`;
    }
}

await Promise.all(
    chunks.map(convertImages)
)





