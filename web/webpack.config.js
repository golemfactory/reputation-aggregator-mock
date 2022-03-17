const path = require('path');
const {CleanWebpackPlugin} = require('clean-webpack-plugin');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const MiniCssExtractPlugin = require('mini-css-extract-plugin');
const WebpackMd5Hash = require('webpack-md5-hash');

module.exports = (env) => ({

    mode: env.production ? 'production' : 'development',

    entry: './src/index.tsx',

    resolve: {
        // Add '.ts' and '.tsx' as resolvable extensions.
        extensions: ['.ts', '.tsx', '.js', '.jsx'],
    },

    output: {
        filename: 'main.js',
        path: path.resolve(__dirname, 'dist', 'bundle'),
    },

    module: {
        rules: [
            {
                test: /\.ts(x?)$/,
                exclude: /node_modules/,
                use: [
                    {
                        loader: 'ts-loader'
                    }
                ]
            },
            {
                test: /\.scss$/,
                use: [
                    'style-loader',
                    {
                        loader: MiniCssExtractPlugin.loader,
                        options: {
                            esModule: false
                        }


                        // options: {
                        //     hmr: !!env.development, // only enable hot in development
                        //     // if hmr does not work, this is a forceful method
                        //     reloadAll: !!env.development
                        // }
                    },
                    'css-loader',
                    {
                        loader: 'postcss-loader', // Run post css actions
                        options: {
                            plugins: () => [
                                require('precss'),
                                require('autoprefixer')
                            ]
                        }
                    },
                    'sass-loader' // compiles Sass to CSS
                ]
            }
        ]
    },
    plugins: [
        new CleanWebpackPlugin(),
        new MiniCssExtractPlugin({
            // Options similar to the same options in webpackOptions.output
            // both options are optional
            filename: env.production ? 'main.[contenthash].css' : 'main.css'
        }),
        new HtmlWebpackPlugin({
            inject: false,
            title: 'actix-web-static-files WebPack',
            hash: true,
            template: './index.html',
            filename: 'index.html'
        })
    ],
    devServer: {
        port: 9000,
        proxy: {
            '/provider': 'http://localhost:5555',
            '/requestor': 'http://localhost:5555',
        },
    },

});
