// Import system prompt from generated source (synchronized with Rust canonical source)
export { REPOBOX_SYSTEM_PROMPT } from './generated-prompt';

export const VENICE_ENDPOINT = "https://api.venice.ai/api/v1/chat/completions";
export const VENICE_MODEL = "llama-3.3-70b"; // Fast on Venice, good at following repo.box YAML format

export const EXPLAIN_EXAMPLES = [
  `groups:
  founders:
    - evm:0x7777777777777777777777777777777777777777
  agents:
    - evm:0x6666666666666666666666666666666666666666

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
    - evm:0x7777777777777777777777777777777777777777
  viewers:
    - evm:0x6666666666666666666666666666666666666666

permissions:
  default: deny
  rules:
    - maintainers own >*
    - viewers read >*`,

  `groups:
  orchestrator:
    - evm:0x8888888888888888888888888888888888888888
  workers:
    - evm:0x9999999999999999999999999999999999999999
    - evm:0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
    - evm:0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
    - evm:0x5555555555555555555555555555555555555555

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
