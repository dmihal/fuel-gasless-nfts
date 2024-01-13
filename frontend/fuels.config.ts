import { createConfig } from 'fuels';

export default createConfig({
  workspace: './..',
  output: './fuel',
  useBuiltinForc: true,
  useBuiltinFuelCore: true,
  autoStartFuelCore: true,
});

/**
 * Check the docs:
 * https://fuellabs.github.io/fuels-ts/guide/cli/config-file
 */
