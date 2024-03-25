import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';
import path from 'node:path';

console.log('Brick location:', path.resolve(__dirname, '../../Brick.ttl'));

export default defineConfig({
	plugins: [sveltekit(), wasm()],
	resolve: {
		alias: {
			'@brick': path.resolve(__dirname, '../../Brick.ttl') + '?raw'
		}
	},
	build: {
		target: 'ES2022'
	}
});
