import { invoke } from "@tauri-apps/api/core"
import { toast } from "sonner"

export type Slot = { code: number | null; script: string | null }
export type Config = { device: string; slots: Slot[] }
export type State = {
  config: Config
  key_names: (string | null)[]
  executable: boolean[]
  devices: string[]
  device_present: boolean
  access_ok: boolean
  daemon_running: boolean
  autostart: boolean
  home: string
  log_path: string
}
export type Learned = { code: number; name: string }

/** Invokes a Tauri command; on failure shows the error as a toast and resolves to undefined. */
export async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T | undefined> {
  try {
    return await invoke<T>(cmd, args)
  } catch (e) {
    toast.error(String(e))
    return undefined
  }
}

export const basename = (p: string) => p.split("/").pop() ?? p
export const dirname = (p: string) => p.slice(0, p.lastIndexOf("/")) || "/"
