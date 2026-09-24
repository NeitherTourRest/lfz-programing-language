/**
 * offpeak —— DeepSeek 错峰省钱闸门（opencode 插件，自动发现于 .opencode/plugin/）
 *
 * 作用：在 **高峰时段** 阻止「派发子智能体」的工具调用（`task` / `call_omo_agent`），
 *       把这些最耗 token 的动作压到 DeepSeek **错峰优惠窗口**内执行。
 *
 * 可配置（环境变量，改后需重启 opencode）：
 *   LFZ_OFFPEAK_START      低谷窗口开始，HH:mm（默认 "00:30"）
 *   LFZ_OFFPEAK_END        低谷窗口结束，HH:mm（默认 "08:30"）
 *   LFZ_OFFPEAK_UTC_OFFSET 时区偏移小时（默认 8 = 北京时间 UTC+8）
 *   LFZ_OFFPEAK_ENFORCE    "0" 表示关闭强制拦截（只放行），默认开启
 *
 * 未在低谷窗口时，被拦截的工具会收到一条明确错误，说明原因与距窗口开启的时间。
 * 闸门只拦「派发子智能体」；读写/构建/测试等本地动作不受影响。
 *
 * 本文件**不 import 任何包**（避免项目内解析不到类型包导致加载失败）。
 */
const START_STR = process.env.LFZ_OFFPEAK_START ?? "00:30"
const END_STR = process.env.LFZ_OFFPEAK_END ?? "08:30"
const UTC_OFFSET = Number(process.env.LFZ_OFFPEAK_UTC_OFFSET ?? 8)
const ENFORCE = (process.env.LFZ_OFFPEAK_ENFORCE ?? "1") !== "0"

/** 需要闸门的工具：所有「会派发子智能体 / 消耗额外模型调用」的入口。 */
const GATED_TOOLS = new Set(["task", "call_omo_agent"])

function toMinutes(hm: string): number {
  const parts = hm.split(":")
  const h = Number.parseInt(parts[0] ?? "0", 10) || 0
  const m = Number.parseInt(parts[1] ?? "0", 10) || 0
  return h * 60 + m
}

export function offpeakStatus(now: Date = new Date()): {
  inWindow: boolean
  minutesUntilChange: number
} {
  const start = toMinutes(START_STR)
  const end = toMinutes(END_STR)
  const local = new Date(now.getTime() + UTC_OFFSET * 3_600_000)
  const mins = local.getUTCHours() * 60 + local.getUTCMinutes()

  const inWindow =
    start < end ? mins >= start && mins < end : mins >= start || mins < end

  let minutesUntilChange = inWindow ? end - mins : start - mins
  if (minutesUntilChange <= 0) minutesUntilChange += 1440

  return { inWindow, minutesUntilChange }
}

function humanize(mins: number): string {
  return `${Math.floor(mins / 60)} 小时 ${mins % 60} 分`
}

export default (async () => {
  return {
    "tool.execute.before": async (input: { tool: string }) => {
      if (!GATED_TOOLS.has(input.tool)) return
      if (!ENFORCE) return

      const { inWindow, minutesUntilChange } = offpeakStatus()
      if (inWindow) return

      throw new Error(
        [
          `[offpeak] 现在是 DeepSeek 高峰时段，已按「低谷开工、高峰停工」策略阻止「${input.tool}」（派发子智能体）。`,
          `低谷窗口：${START_STR}–${END_STR}（UTC+${UTC_OFFSET}）；距今约 ${humanize(minutesUntilChange)} 开启。`,
          `只拦「派发」，本地读写/构建/测试不受影响；确需高峰开工可设 LFZ_OFFPEAK_ENFORCE=0 后重启 opencode。`,
        ].join("\n"),
      )
    },
  }
})()
