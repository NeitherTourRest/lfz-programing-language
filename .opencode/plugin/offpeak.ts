/**
 * offpeak -- DeepSeek off-peak work gate (opencode plugin, auto-discovered in .opencode/plugin/).
 *
 * OFFICIAL (api-docs.deepseek.com/quick_start/pricing):
 *   Peak = Mon-Fri 01:00-04:00 and 06:00-10:00 UTC, excluding Chinese public holidays.
 *   All other hours are OFF-PEAK (50% price).
 *   Beijing (UTC+8): peak = Mon-Fri 09:00-12:00 and 14:00-18:00.
 *
 * Behaviour: during PEAK hours this plugin BLOCKS subagent dispatch
 * (`task` / `call_omo_agent`) so the most token-hungry work is pushed into the
 * discounted window. Local reads/builds/tests/commits are unaffected.
 *
 * Config via env (restart opencode after changing):
 *   LFZ_PEAK_UTC="01:00-04:00,06:00-10:00"   peak ranges in UTC
 *   LFZ_PEAK_DAYS="1-5"                      day-of-week set (0=Sun .. 6=Sat)
 *   LFZ_HOLIDAYS="2026-10-01,2026-10-02"     dates (UTC==Beijing for these windows)
 *   LFZ_OFFPEAK_ENFORCE="0"                  disable enforcement
 *
 * No imports on purpose (avoids module-resolution failures inside the project).
 */
const PEAK_SPEC = process.env.LFZ_PEAK_UTC ?? "01:00-04:00,06:00-10:00"
const DAY_SPEC = process.env.LFZ_PEAK_DAYS ?? "1-5"
const HOLIDAYS = (process.env.LFZ_HOLIDAYS ?? "")
  .split(",")
  .map((s) => s.trim())
  .filter((s) => s !== "")
const ENFORCE = (process.env.LFZ_OFFPEAK_ENFORCE ?? "1") !== "0"

const PEAK_MINUTES: Array<[number, number]> = PEAK_SPEC.split(",")
  .map((r) => r.trim())
  .filter((r) => r !== "")
  .map((r) => {
    const [a, b] = r.split("-")
    const toMin = (hm: string) => {
      const [h, m] = hm.split(":")
      return (parseInt(h ?? "0", 10) || 0) * 60 + (parseInt(m ?? "0", 10) || 0)
    }
    return [toMin(a ?? "0:0"), toMin(b ?? "0:0")] as [number, number]
  })

const PEAK_DAYS = new Set<number>()
for (const part of DAY_SPEC.split(",")) {
  const p = part.trim()
  if (p === "") continue
  if (p.includes("-")) {
    const [a, b] = p.split("-").map((x) => parseInt(x, 10))
    for (let i = a; i <= b; i++) PEAK_DAYS.add(i)
  } else {
    PEAK_DAYS.add(parseInt(p, 10))
  }
}

function pad(n: number): string {
  return n < 10 ? `0${n}` : `${n}`
}

function isPeak(t: Date): boolean {
  if (!PEAK_DAYS.has(t.getUTCDay())) return false
  const dateStr = `${t.getUTCFullYear()}-${pad(t.getUTCMonth() + 1)}-${pad(t.getUTCDate())}`
  if (HOLIDAYS.includes(dateStr)) return false
  const m = t.getUTCHours() * 60 + t.getUTCMinutes()
  return PEAK_MINUTES.some(([a, b]) => m >= a && m < b)
}

export function offpeakStatus(now: Date = new Date()): { inPeak: boolean; minutesUntilChange: number } {
  const inPeak = isPeak(now)
  let minutesUntilChange = 0
  for (let k = 0; k < 10080; k++) {
    if (isPeak(new Date(now.getTime() + (k + 1) * 60_000)) !== inPeak) {
      minutesUntilChange = k + 1
      break
    }
  }
  return { inPeak, minutesUntilChange }
}

function humanize(mins: number): string {
  return `${Math.floor(mins / 60)}h ${mins % 60}m`
}

const GATED_TOOLS = new Set(["task", "call_omo_agent"])

export default (async () => {
  return {
    "tool.execute.before": async (input: { tool: string }) => {
      if (!GATED_TOOLS.has(input.tool)) return
      if (!ENFORCE) return

      const { inPeak, minutesUntilChange } = offpeakStatus()
      if (!inPeak) return

      throw new Error(
        [
          `[offpeak] 现在是 DeepSeek 高峰时段，已按省钱策略阻止「${input.tool}」（派发子智能体）。`,
          `高峰 = 周一至周五 ${PEAK_SPEC} UTC（北京 09:00-12:00 与 14:00-18:00）；距低谷开始约 ${humanize(minutesUntilChange)}。`,
          `只拦「派发」，本地读写/构建/测试/提交不受影响；确需高峰开工可设 LFZ_OFFPEAK_ENFORCE=0 后重启 opencode。`,
        ].join("\n"),
      )
    },
  }
})()
