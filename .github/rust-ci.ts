import { createRustWorkflow } from '@dougefresh/ci';

export default function () {
  return createRustWorkflow().semver(false).extra('cli-test', 'bash -x ./scripts/test-ci.sh').build();
}
