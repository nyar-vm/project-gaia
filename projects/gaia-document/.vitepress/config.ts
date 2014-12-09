import {defineConfig} from 'vitepress';
import msilGrammar from './msil.tmLanguage.json' with {type: 'json'}
import jasmGrammar from './jasm.tmLanguage.json' with {type: 'json'}
import valkyrieGrammar from './valkyrie.tmLanguage.json' with {type: 'json'}
import mermaidPlugin from "./mermaid-plugin";

export default defineConfig({
    title: 'Gaia Assembler',
    description: 'Gaia - 现代多平台汇编器和工具链',

    markdown: {
        theme: {
            light: 'one-light',
            dark: 'one-dark-pro'
        },
        config: (md) => {
            console.log('config.ts: markdown.config function called');
            md.use(mermaidPlugin);
        },
        shikiSetup(shiki) {
            shiki.loadLanguageSync({
                name: 'msil',
                scopeName: 'source.msil',
                fileTypes: ['msil'],
                patterns: msilGrammar.patterns,
                repository: msilGrammar.repository
            })
            shiki.loadLanguageSync({
                name: 'jasm',
                scopeName: 'source.jasm',
                fileTypes: ['jasm'],
                patterns: jasmGrammar.patterns,
                repository: jasmGrammar.repository
            })
            shiki.loadLanguageSync({
                name: 'gaia',
                scopeName: 'source.valkyrie',
                fileTypes: ['gaia'],
                patterns: valkyrieGrammar.patterns,
                repository: valkyrieGrammar.repository
            })
        }
    },
    themeConfig: {
        logo: '/logo.svg',
        nav: [
            {text: '首页', link: '/'},
            {
                text: '用户文档',
                items: [
                    {text: '快速开始', link: '/getting-started/'},
                    {text: '用户指南', link: '/user-guide/'},
                    {text: '后端支持', link: '/backends/'}
                ]
            },
            {
                text: '开发者文档',
                items: [
                    {text: '开发者指南', link: '/developer-guide/'},
                    {text: 'API 参考', link: '/api-reference/'}
                ]
            },
            {
                text: '后端支持',
                items: [
                    {text: '.NET (C#)', link: '/backends/clr/'},
                    {text: 'JVM (Java)', link: '/backends/jvm/'},
                    {text: 'PE (Windows)', link: '/backends/pe/'},
                    {text: 'ELF (Linux/Unix)', link: '/backends/elf/'},
                    {text: 'WASM (WebAssembly)', link: '/backends/wasm/'},
                    {text: 'LUAC (Lua)', link: '/backends/lua/'},
                    {text: 'PYC (Python)', link: '/backends/pyc/'},
                ]
            },
        ],
        sidebar: {
            '/getting-started/': [
                {
                    text: '快速开始',
                    items: [
                        {text: '概述', link: '/getting-started/'},
                        {text: '第一个程序', link: '/getting-started/first-program'},
                        {text: '核心概念', link: '/getting-started/concepts'},
                        {text: '示例代码', link: '/getting-started/examples'}
                    ]
                }
            ],
            '/user-guide/': [
                {
                    text: '用户指南',
                    items: [
                        {text: '概述', link: '/user-guide/'},
                        {text: '接口使用', link: '/user-guide/interface-usage'},
                        {text: '集成指南', link: '/user-guide/integration'},
                        {text: '调试指南', link: '/user-guide/debugging'},
                        {text: '最佳实践', link: '/user-guide/best-practices'}
                    ]
                }
            ],
            '/developer-guide/': [
                {
                    text: '开发者指南',
                    items: [
                        {text: '概述', link: '/developer-guide/'},
                        {text: '项目架构', link: '/developer-guide/architecture'},
                        {text: '前端开发', link: '/developer-guide/frontend-development'},
                        {text: '后端开发', link: '/developer-guide/backend-development'},
                        {text: '语言服务器', link: '/developer-guide/language-server'},
                        {text: '测试指南', link: '/developer-guide/testing'},
                        {text: '贡献指南', link: '/developer-guide/contributing'}
                    ]
                }
            ],
            '/api-reference/': [
                {
                    text: 'API 参考',
                    items: [
                        {text: '概述', link: '/api-reference/'},
                        {text: 'gaia-assembler', link: '/api-reference/gaia-assembler'},
                        {text: 'gaia-types', link: '/api-reference/gaia-types'},
                        {text: 'gaia-frontend', link: '/api-reference/gaia-frontend'},
                        {text: '后端 API', link: '/api-reference/backends'}
                    ]
                }
            ],

            '/maintenance/': [
                {
                    text: '维护指南',
                    items: [
                        {text: '概述', link: '/maintenance/'},
                        {text: '发布流程', link: '/maintenance/release-process'},
                        {text: '安全指南', link: '/maintenance/security'},
                        {text: '故障排除', link: '/maintenance/troubleshooting'}
                    ]
                }
            ],
            '/backends/': [
                {
                    text: '后端支持',
                    items: [
                        {text: '后端概述', link: '/backends/'},
                        {text: 'CLR (.NET)', link: '/backends/clr/'},
                        {text: 'CLASS (JVM)', link: '/backends/jvm/'},
                        {text: 'PE (Windows)', link: '/backends/pe/'},
                        {text: 'ELF (Linux/Unix)', link: '/backends/elf/'},
                        {text: 'WASM (WebAssembly)', link: '/backends/wasm/'},
                        {text: 'LUAC (Lua)', link: '/backends/lua/'},
                        {text: 'PYC (Python)', link: '/backends/pyc/'},
                        {text: 'Gaia Assembly', link: '/backends/gaia/'}
                    ]
                }
            ],
            '/backends/clr/': [
                {
                    text: 'CLR/MSIL 指令',
                    items: [
                        {text: 'MSIL 概述', link: '/backends/clr/'},
                        {text: '基础指令', link: '/backends/clr/basic-instructions'},
                        {text: '算术指令', link: '/backends/clr/arithmetic-instructions'},
                        {text: '控制流指令', link: '/backends/clr/control-flow-instructions'},
                        {text: '方法调用指令', link: '/backends/clr/method-instructions'},
                        {text: '对象操作指令', link: '/backends/clr/object-instructions'},
                        {text: '异常处理指令', link: '/backends/clr/exception-instructions'}
                    ]
                }
            ],
            '/backends/jvm/': [
                {
                    text: 'JVM/JASM 指令',
                    items: [
                        {text: 'JASM 概述', link: '/backends/jvm/'},
                        {text: '基础指令', link: '/backends/jvm/basic-instructions'},
                        {text: '算术指令', link: '/backends/jvm/arithmetic-instructions'},
                        {text: '控制流指令', link: '/backends/jvm/control-flow-instructions'},
                        {text: '方法调用指令', link: '/backends/jvm/method-instructions'},
                        {text: '对象操作指令', link: '/backends/jvm/object-instructions'},
                        {text: '异常处理指令', link: '/backends/jvm/exception-instructions'}
                    ]
                }
            ],
            '/backends/pe/': [
                {
                    text: 'PE (Windows)',
                    items: [
                        {text: 'PE 概述', link: '/backends/pe/'},
                        {text: '基本概念', link: '/backends/pe/concepts'},
                        {text: '文件结构', link: '/backends/pe/file-structure'},
                        {text: '入门指南', link: '/backends/pe/getting-started'}
                    ]
                }
            ],
            '/backends/elf/': [
                {
                    text: 'ELF (Linux/Unix)',
                    items: [
                        {text: 'ELF 概述', link: '/backends/elf/'},
                        {text: '基本概念', link: '/backends/elf/concepts'},
                        {text: '文件结构', link: '/backends/elf/file-structure'},
                        {text: '入门指南', link: '/backends/elf/getting-started'}
                    ]
                }
            ],
            '/backends/wasm/': [
                {
                    text: 'WASM (WebAssembly)',
                    items: [
                        {text: 'WASM 概述', link: '/backends/wasm/'},
                        {text: '基本概念', link: '/backends/wasm/concepts'},
                        {text: '入门指南', link: '/backends/wasm/getting-started'},
                        {text: '模块结构', link: '/backends/wasm/module-structure'}
                    ]
                }
            ],
            '/backends/lua/': [
                {
                    text: 'LUAC (Lua)',
                    items: [
                        {text: 'Lua 概述', link: '/backends/lua/'},
                        {text: '基础指令', link: '/backends/lua/basic-instructions'},
                        {text: '算术指令', link: '/backends/lua/arithmetic-instructions'},
                        {text: '控制流指令', link: '/backends/lua/control-flow-instructions'},
                        {text: '函数调用指令', link: '/backends/lua/method-instructions'},
                        {text: '对象操作指令', link: '/backends/lua/object-instructions'},
                        {text: '异常处理指令', link: '/backends/lua/exception-instructions'}
                    ]
                }
            ],
            '/backends/pyc/': [
                {
                    text: 'PYC (Python)',
                    items: [
                        {text: 'Python 概述', link: '/backends/pyc/'},
                        {text: '基础指令', link: '/backends/pyc/basic-instructions'},
                        {text: '算术指令', link: '/backends/pyc/arithmetic-instructions'},
                        {text: '控制流指令', link: '/backends/pyc/control-flow-instructions'},
                        {text: '函数调用指令', link: '/backends/pyc/method-instructions'},
                        {text: '对象操作指令', link: '/backends/pyc/object-instructions'},
                        {text: '异常处理指令', link: '/backends/pyc/exception-instructions'}
                    ]
                }
            ]
        },

        socialLinks: [
            {icon: 'github', link: 'https://github.com/nyar-vm/project-gaia'}
        ],

        footer: {
            message: 'Released under the MIT License.',
            copyright: 'Copyright © 2024 Gaia Project'
        }
    },
    vite: {
        ssr: {noExternal: ['dayjs']},
        optimizeDeps: {include: ['@braintree/sanitize-url']}
    },
    head: [
        // ['script', {src: 'https://cdn.jsdelivr.net/npm/mermaid@10.9.1/dist/mermaid.min.js'}],
        // ['script', {}, 'window.addEventListener(\'load\', () => { mermaid.initialize({ startOnLoad: true }); });']
    ]
});