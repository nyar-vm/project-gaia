import {defineConfig} from 'vitest/config';

export default defineConfig({
    test: {
        globals: true,
        environment: 'node',
        include: ['tests/**/*.test.ts'],
        coverage: {
            provider: 'v8',
            reporter: ['text', 'html', 'lcov'],
            exclude: [
                '**/dist/**',
                '**/node_modules/**',
                '**/tests/**'
            ]
        },
        timeout: 30000 // 增加超时时间，因为 WASM 加载可能较慢
    },
    resolve: {
        alias: {
            '@': './dist'
        }
    }
});