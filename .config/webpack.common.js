const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');

const dist = path.resolve(__dirname, '..', 'docs');

module.exports = {
  entry: './index.js',
  output: {
    path: dist,
    filename: '[name].js',
  },
  module: {
    rules: [
      {
        test: /\.?js$/,
        exclude: /node_modules/,
        use: {
          loader: 'babel-loader',
          options: {
            presets: [
              [
                '@babel/preset-env', 
                {
                  exclude: [
                    '@babel/plugin-transform-regenerator',
                  ],
                  'useBuiltIns': false,
                }
              ],
              '@babel/preset-react'
            ],
          }
        }
      },
      {
        test: /\.css$/, 
        use: ['style-loader', 'css-loader'],
      },
      {test: /\.svg$/, loader: 'file-loader'},
    ]
  },
  plugins: [
    new HtmlWebpackPlugin({
      title: 'Hades Save Editor'
    })
  ],
  experiments: {
    asyncWebAssembly: true,
  }
};