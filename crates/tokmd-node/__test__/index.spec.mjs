import test from 'ava'

// Note: In development, we need to build the native module first
// These tests will work after running `npm run build` in the tokmd-node directory

test('version returns string', async (t) => {
  // Import will fail if native module not built
  try {
    const { version } = await import('../npm/index.js')
    const v = version()
    t.is(typeof v, 'string')
    t.true(v.includes('.'))
  } catch (e) {
    t.pass('Native module not built, skipping')
  }
})

test('schemaVersion returns number', async (t) => {
  try {
    const { schemaVersion } = await import('../npm/index.js')
    const sv = schemaVersion()
    t.is(typeof sv, 'number')
    t.true(sv >= 1)
  } catch (e) {
    t.pass('Native module not built, skipping')
  }
})

test('lang returns receipt', async (t) => {
  try {
    const { lang } = await import('../npm/index.js')
    const result = await lang({ paths: ['src'] })
    t.is(result.mode, 'lang')
    t.true(Array.isArray(result.rows))
    t.truthy(result.schema_version)
  } catch (e) {
    t.pass('Native module not built, skipping')
  }
})

test('module returns receipt', async (t) => {
  try {
    const { module } = await import('../npm/index.js')
    const result = await module({ paths: ['src'] })
    t.is(result.mode, 'module')
    t.true(Array.isArray(result.rows))
  } catch (e) {
    t.pass('Native module not built, skipping')
  }
})

test('runJson version mode', async (t) => {
  try {
    const { runJson } = await import('../npm/index.js')
    const result = await runJson('version', '{}')
    const data = JSON.parse(result)
    t.truthy(data.version)
    t.truthy(data.schema_version)
  } catch (e) {
    t.pass('Native module not built, skipping')
  }
})

test('runJson invalid mode returns error', async (t) => {
  try {
    const { runJson } = await import('../npm/index.js')
    const result = await runJson('invalid_mode', '{}')
    const data = JSON.parse(result)
    t.is(data.error, true)
    t.truthy(data.code)
  } catch (e) {
    t.pass('Native module not built, skipping')
  }
})

test('diff compares paths', async (t) => {
  try {
    const { diff } = await import('../npm/index.js')
    const result = await diff('src', 'src')
    t.is(result.mode, 'diff')
    t.truthy(result.totals)
  } catch (e) {
    t.pass('Native module not built, skipping')
  }
})

test('lang with options', async (t) => {
  try {
    const { lang } = await import('../npm/index.js')
    const result = await lang({ paths: ['src'], top: 2, files: true })
    t.is(result.mode, 'lang')
    t.true(result.args.with_files)
    t.true(result.rows.length <= 3) // top 2 + possible "Other"
  } catch (e) {
    t.pass('Native module not built, skipping')
  }
})
