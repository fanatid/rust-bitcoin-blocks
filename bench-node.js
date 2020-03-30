#!/usr/bin/env node
const bjs = require('bitcoinjs-lib')
const { readFileSync } = require('fs')

function measure(text, iters, fn) {
  const elapsed = []
  for (let i = 0; i < iters; ++i) {
    const ts = process.hrtime()
    fn()
    const t = process.hrtime(ts)
    elapsed.push(t[0] * 1e3 + t[1] * 1e-6)
  }

  elapsed.sort((a, b) => a - b)
  console.log(`Parse ${text} (${iters} iterations):`)
  console.log(`min: ${elapsed[0].toFixed(3)}ms`)
  console.log(`average: ${(elapsed.reduce((a, b) => a + b) / elapsed.length).toFixed(3)}ms`)
  console.log(`max: ${elapsed[elapsed.length - 1].toFixed(3)}ms`)
}

// JSON
const data = readFileSync('./blocks/623200.json', 'utf8')
measure('JSON', 100, () => JSON.parse(data))

// HEX
const data_hex = readFileSync('./blocks/623200.hex', 'utf8').trim()
measure('HEX', 100, () => {
  const data_buf = Buffer.from(data_hex, 'hex')
  bjs.Block.fromBuffer(data_buf)
})
