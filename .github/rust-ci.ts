import { createRustWorkflow } from '@dougefresh/ci';

export default function () {
  return createRustWorkflow().semver(false).build();
}
