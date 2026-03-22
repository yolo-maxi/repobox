// Import system prompt from generated source (synchronized with Rust canonical source)
export { REPOBOX_SYSTEM_PROMPT } from './generated-prompt';

export const VENICE_ENDPOINT = "https://api.venice.ai/api/v1/chat/completions";
export const VENICE_MODEL = "llama-3.3-70b"; // Fast on Venice, good at following repo.box YAML format

export const EXPLAIN_EXAMPLES = [
  `groups:
  founders:
    - evm:0xAAA...123
  agents:
    - evm:0xBBB...456

permissions:
  default: allow
  rules:
    - founders own >*
    - agents:
        push:
          - ">feature/**"
        branch:
          - ">feature/**"
        create:
          - ">feature/**"`,

  `groups:
  maintainers:
    - evm:0xAAA...123
  viewers:
    - evm:0xBBB...456

permissions:
  default: deny
  rules:
    - maintainers own >*
    - viewers read >*`,

  `groups:
  orchestrator:
    - evm:0xAAA...001
  workers:
    - evm:0xBBB...002
    - evm:0xCCC...003
    - evm:0xDDD...004
    - evm:0xEEE...005

permissions:
  default: deny
  rules:
    - orchestrator own >*
    - workers not force-push >*
    - workers:
        push:
          - ">worker-1/**"
          - ">worker-2/**" 
          - ">worker-3/**"
          - ">worker-4/**"
        branch:
          - ">worker-1/**"
          - ">worker-2/**"
          - ">worker-3/**" 
          - ">worker-4/**"
        create:
          - ">worker-1/**"
          - ">worker-2/**"
          - ">worker-3/**"
          - ">worker-4/**"
        edit:
          - "./* >worker-1/**"
          - "./* >worker-2/**"
          - "./* >worker-3/**"
          - "./* >worker-4/**"
        append:
          - "./.repobox/config.yml"`,
];
