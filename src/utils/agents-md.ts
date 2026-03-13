export const OMX_GENERATED_AGENTS_MARKER = '<!-- omx:generated:agents-md -->'
const OMX_GENERATED_AGENTS_TITLE =
  '# oh-my-codex - Intelligent Multi-Agent Orchestration'

export function isOmxGeneratedAgentsMd(content: string): boolean {
  return (
    content.includes(OMX_GENERATED_AGENTS_MARKER) ||
    content.includes(OMX_GENERATED_AGENTS_TITLE)
  )
}

export function addGeneratedAgentsMarker(content: string): string {
  if (content.includes(OMX_GENERATED_AGENTS_MARKER)) return content

  const firstNewline = content.indexOf('\n')
  if (firstNewline === -1) {
    return `${content}\n${OMX_GENERATED_AGENTS_MARKER}\n`
  }

  return (
    content.slice(0, firstNewline + 1) +
    `${OMX_GENERATED_AGENTS_MARKER}\n` +
    content.slice(firstNewline + 1)
  )
}
