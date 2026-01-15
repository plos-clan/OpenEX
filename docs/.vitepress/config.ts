import {defineConfig} from 'vitepress'

export default defineConfig({
    base: "/OpenEX/",
    title: "OpenEX",
    description: "OpenEX RustEdition",
    themeConfig: {
        logo: "/icon.png",
        nav: [
            {text: 'Home', link: '/'},
            {text: 'Examples', link: '/example/'}
        ],

        sidebar: [
            {
                text: 'Started',
                items: [
                    {text: '简介', link: '/started'},
                    {text: '命令行参数', link: '/started/argument'},
                    {text: '优化选项', link: '/started/optimizer'},
                ]
            },
            {
                text: '基础教程',
                items: [
                    {text: '第一个程序', link: '/example'},
                    {text: '依赖导入', link: '/example/import'},
                    {text: '表达式', link: '/example/expression'},
                    {text: '函数', link: '/example/function'},
                    {text: '判断语句', link: '/example/conditional'},
                    {text: '循环语句', link: '/example/loop'},
                    {text: '变量', link: '/example/value'},
                    {text: '线程安全', link: '/example/sync'},
                ]
            },
            {
                text: 'Runtime API',
                items: [
                    {text: 'Index', link: '/api'},
                    {text: 'system', link: '/api/system'},
                    {text: 'math', link: '/api/math'},
                    {text: 'type', link: '/api/type'},
                    {text: 'cffi', link: '/api/cffi'},
                ]
            },
            {
                text: 'OpenEX IR',
                items: [
                    {text: 'Index', link: '/bytecode'},
                    {text: '操作类型指令', link: '/bytecode/vm_table_opt'},
                    {text: '运算类型指令', link: '/bytecode/vm_operation'},
                ]
            }
        ],

        socialLinks: [
            {icon: 'github', link: 'https://github.com/plos-clan/OpenEX'}
        ],
        footer: {
            message: "本文档采用 知识共享 署名-相同方式共享 4.0 协议 进行许可。",
            copyright: "Copyright © 2023-2026 MCPPL,DotCS",
        },
    },
    vite: {
        server: {
            watch: {
                usePolling: true,
                interval: 100
            }
        },
    },
    markdown: {
        lineNumbers: true,
        math: true,
    }
})
