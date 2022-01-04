const path = require("path");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const { merge } = require('webpack-merge')
const common = require('./webpack.common.js')

const crate = path.resolve(__dirname, '..');

module.exports = merge(common, {
  mode: 'production',
  devtool: 'source-map',
  plugins: [
    new WasmPackPlugin({
      crateDirectory: crate,
      extraArgs: '-- --features web',
    }),
  ]
});