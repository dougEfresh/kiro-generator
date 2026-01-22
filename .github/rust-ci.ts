import { createRustWorkflow } from '@carteramesh/ci';

export default function () {
  return createRustWorkflow().semver(false).build();
}
