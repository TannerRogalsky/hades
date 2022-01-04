const path = require('path');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const { merge } = require('webpack-merge')
const common = require('./webpack.common.js')

const crate = path.resolve(__dirname, '..');

module.exports = merge(common, {
  mode: 'development',
  devtool: 'eval',
  devServer: {
    historyApiFallback: true,
    compress: true,
    hot: true,
    port: 8080,
  },
  plugins: [
    new WasmPackPlugin({
      crateDirectory: crate,
      forceMode: 'production',
      extraArgs: '--profiling -- --features web',
    }),
  ],
  experiments: {
    asyncWebAssembly: true,
  }
});