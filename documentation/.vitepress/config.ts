import {defineConfig} from 'vitepress';
import msilGrammar from './msil.tmLanguage.json' with {type: 'json'}
import jasmGrammar from './jasm.tmLanguage.json' with {type: 'json'}
import valkyrieGrammar from './valkyrie.tmLanguage.json' with {type: 'json'}
import mermaidPlugin from "./mermaid-plugin";

export default defineConfig({
    title: 'Gaia Assembler',
    description: 'Gaia - 现代多平台汇编器和工具链',

    locales: {
        root: {
            label: '简体中文',
            lang: 'zh-Hans',
            link: '/zh-hans/',
            themeConfig: {
                nav: [
                    {text: '首页', link: '/zh-hans/'},
                    {
                        text: '用户文档',
                        items: [
                            {text: '快速开始', link: '/zh-hans/getting-started/'},
                            {text: '用户指南', link: '/zh-hans/user-guide/'},
                            {text: '后端支持', link: '/zh-hans/backends/'}
                        ]
                    },
                    {
                        text: '开发者文档',
                        items: [
                            {text: '开发者指南', link: '/zh-hans/developer-guide/'},
                            {text: 'API 参考', link: '/zh-hans/api-reference/'}
                        ]
                    },
                    {
                        text: '后端支持',
                        items: [
                            {text: '.NET (C#)', link: '/zh-hans/backends/clr/'},
                            {text: 'JVM (Java)', link: '/zh-hans/backends/jvm/'},
                            {text: 'PE (Windows)', link: '/zh-hans/backends/pe/'},
                            {text: 'ELF (Linux/Unix)', link: '/zh-hans/backends/elf/'},
                            {text: 'WASM (WebAssembly)', link: '/zh-hans/backends/wasm/'},
                            {text: 'LUAC (Lua)', link: '/zh-hans/backends/lua/'},
                            {text: 'PYC (Python)', link: '/zh-hans/backends/pyc/'},
                        ]
                    },
                ],
                sidebar: {
                    '/zh-hans/getting-started/': [
                        {
                            text: '快速开始',
                            items: [
                                {text: '概述', link: '/zh-hans/getting-started/'},
                                {text: '第一个程序', link: '/zh-hans/getting-started/first-program'},
                                {text: '核心概念', link: '/zh-hans/getting-started/concepts'},
                                {text: '示例代码', link: '/zh-hans/getting-started/examples'}
                            ]
                        }
                    ],
                    '/zh-hans/user-guide/': [
                        {
                            text: '用户指南',
                            items: [
                                {text: '概述', link: '/zh-hans/user-guide/'},
                                {text: '接口使用', link: '/zh-hans/user-guide/interface-usage'},
                                {text: '集成指南', link: '/zh-hans/user-guide/integration'},
                                {text: '调试指南', link: '/zh-hans/user-guide/debugging'},
                                {text: '最佳实践', link: '/zh-hans/user-guide/best-practices'}
                            ]
                        }
                    ],
                    '/zh-hans/developer-guide/': [
                        {
                            text: '开发者指南',
                            items: [
                                {text: '概述', link: '/zh-hans/developer-guide/'},
                                {text: '项目架构', link: '/zh-hans/developer-guide/architecture'},
                                {text: '前端开发', link: '/zh-hans/developer-guide/frontend-development'},
                                {text: '后端开发', link: '/zh-hans/developer-guide/backend-development'},
                                {text: '语言服务器', link: '/zh-hans/developer-guide/language-server'},
                                {text: '测试指南', link: '/zh-hans/developer-guide/testing'},
                                {text: '贡献指南', link: '/zh-hans/developer-guide/contributing'}
                            ]
                        }
                    ],
                    '/zh-hans/api-reference/': [
                        {
                            text: 'API 参考',
                            items: [
                                {text: '概述', link: '/zh-hans/api-reference/'},
                                {text: 'gaia-assembler', link: '/zh-hans/api-reference/gaia-assembler'},
                                {text: 'gaia-types', link: '/zh-hans/api-reference/gaia-types'},
                                {text: 'gaia-frontend', link: '/zh-hans/api-reference/gaia-frontend'},
                                {text: '后端 API', link: '/zh-hans/api-reference/backends'}
                            ]
                        }
                    ],
                    '/zh-hans/maintenance/': [
                        {
                            text: '维护指南',
                            items: [
                                {text: '概述', link: '/zh-hans/maintenance/'},
                                {text: '发布流程', link: '/zh-hans/maintenance/release-process'},
                                {text: '安全指南', link: '/zh-hans/maintenance/security'},
                                {text: '故障排除', link: '/zh-hans/maintenance/troubleshooting'}
                            ]
                        }
                    ],
                    '/zh-hans/backends/': [
                        {
                            text: '后端支持',
                            items: [
                                {text: '后端概述', link: '/zh-hans/backends/'},
                                {text: 'CLR (.NET)', link: '/zh-hans/backends/clr/'},
                                {text: 'CLASS (JVM)', link: '/zh-hans/backends/jvm/'},
                                {text: 'PE (Windows)', link: '/zh-hans/backends/pe/'},
                                {text: 'ELF (Linux/Unix)', link: '/zh-hans/backends/elf/'},
                                {text: 'WASM (WebAssembly)', link: '/zh-hans/backends/wasm/'},
                                {text: 'LUAC (Lua)', link: '/zh-hans/backends/lua/'},
                                {text: 'PYC (Python)', link: '/zh-hans/backends/pyc/'},
                                {text: 'Gaia Assembly', link: '/zh-hans/backends/gaia/'}
                            ]
                        }
                    ],
                    '/zh-hans/backends/clr/': [
                        {
                            text: 'CLR/MSIL 指令',
                            items: [
                                {text: 'MSIL 概述', link: '/zh-hans/backends/clr/'},
                                {text: '基础指令', link: '/zh-hans/backends/clr/basic-instructions'},
                                {text: '算术指令', link: '/zh-hans/backends/clr/arithmetic-instructions'},
                                {text: '控制流指令', link: '/zh-hans/backends/clr/control-flow-instructions'},
                                {text: '方法调用指令', link: '/zh-hans/backends/clr/method-instructions'},
                                {text: '对象操作指令', link: '/zh-hans/backends/clr/object-instructions'},
                                {text: '异常处理指令', link: '/zh-hans/backends/clr/exception-instructions'}
                            ]
                        }
                    ],
                    '/zh-hans/backends/jvm/': [
                        {
                            text: 'JVM/JASM 指令',
                            items: [
                                {text: 'JASM 概述', link: '/zh-hans/backends/jvm/'},
                                {text: '基础指令', link: '/zh-hans/backends/jvm/basic-instructions'},
                                {text: '算术指令', link: '/zh-hans/backends/jvm/arithmetic-instructions'},
                                {text: '控制流指令', link: '/zh-hans/backends/jvm/control-flow-instructions'},
                                {text: '方法调用指令', link: '/zh-hans/backends/jvm/method-instructions'},
                                {text: '对象操作指令', link: '/zh-hans/backends/jvm/object-instructions'},
                                {text: '异常处理指令', link: '/zh-hans/backends/jvm/exception-instructions'}
                            ]
                        }
                    ],
                    '/zh-hans/backends/pe/': [
                        {
                            text: 'PE (Windows)',
                            items: [
                                {text: 'PE 概述', link: '/zh-hans/backends/pe/'},
                                {text: '基本概念', link: '/zh-hans/backends/pe/concepts'},
                                {text: '文件结构', link: '/zh-hans/backends/pe/file-structure'},
                                {text: '入门指南', link: '/zh-hans/backends/pe/getting-started'}
                            ]
                        }
                    ],
                    '/zh-hans/backends/elf/': [
                        {
                            text: 'ELF (Linux/Unix)',
                            items: [
                                {text: 'ELF 概述', link: '/zh-hans/backends/elf/'},
                                {text: '基本概念', link: '/zh-hans/backends/elf/concepts'},
                                {text: '文件结构', link: '/zh-hans/backends/elf/file-structure'},
                                {text: '入门指南', link: '/zh-hans/backends/elf/getting-started'}
                            ]
                        }
                    ],
                    '/zh-hans/backends/wasm/': [
                        {
                            text: 'WASM (WebAssembly)',
                            items: [
                                {text: 'WASM 概述', link: '/zh-hans/backends/wasm/'},
                                {text: '基本概念', link: '/zh-hans/backends/wasm/concepts'},
                                {text: '入门指南', link: '/zh-hans/backends/wasm/getting-started'},
                                {text: '模块结构', link: '/zh-hans/backends/wasm/module-structure'}
                            ]
                        }
                    ],
                    '/zh-hans/backends/lua/': [
                        {
                            text: 'LUAC (Lua)',
                            items: [
                                {text: 'Lua 概述', link: '/zh-hans/backends/lua/'},
                                {text: '基础指令', link: '/zh-hans/backends/lua/basic-instructions'},
                                {text: '算术指令', link: '/zh-hans/backends/lua/arithmetic-instructions'},
                                {text: '控制流指令', link: '/zh-hans/backends/lua/control-flow-instructions'},
                                {text: '函数调用指令', link: '/zh-hans/backends/lua/method-instructions'},
                                {text: '对象操作指令', link: '/zh-hans/backends/lua/object-instructions'},
                                {text: '异常处理指令', link: '/zh-hans/backends/lua/exception-instructions'}
                            ]
                        }
                    ],
                    '/zh-hans/backends/pyc/': [
                        {
                            text: 'PYC (Python)',
                            items: [
                                {text: 'Python 概述', link: '/zh-hans/backends/pyc/'},
                                {text: '基础指令', link: '/zh-hans/backends/pyc/basic-instructions'},
                                {text: '算术指令', link: '/zh-hans/backends/pyc/arithmetic-instructions'},
                                {text: '控制流指令', link: '/zh-hans/backends/pyc/control-flow-instructions'},
                                {text: '函数调用指令', link: '/zh-hans/backends/pyc/method-instructions'},
                                {text: '对象操作指令', link: '/zh-hans/backends/pyc/object-instructions'},
                                {text: '异常处理指令', link: '/zh-hans/backends/pyc/exception-instructions'}
                            ]
                        }
                    ]
                }
            }
        },
        en: {
            label: 'English',
            lang: 'en-US',
            link: '/en-us/',
            themeConfig: {
                nav: [
                    {text: 'Home', link: '/en-us/'},
                ],
                sidebar: {}
            }
        }
    },

    markdown: {
        theme: {
            light: 'one-light',
            dark: 'one-dark-pro'
        },
        config: (md) => {
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
        optimizeDeps: {include: ['↯braintree/sanitize-url']}
    },
    head: []
});
