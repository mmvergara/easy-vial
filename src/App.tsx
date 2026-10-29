import { useCallback, useEffect, useRef, useState } from "react"
import { open } from "@tauri-apps/plugin-dialog"
import { Keyboard, Play, Power, Square } from "lucide-react"
import { toast } from "sonner"

import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardAction, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Label } from "@/components/ui/label"
import { NativeSelect, NativeSelectOption } from "@/components/ui/native-select"
import { Switch } from "@/components/ui/switch"
import { Toaster } from "@/components/ui/sonner"
import { ChmodDialog } from "@/chmod-dialog"
import { SlotCard } from "@/slot-card"
import { basename, call, dirname, type Learned, type State } from "@/lib/api"

export function App() {
  const [state, setState] = useState<State | null>(null)
  const [learning, setLearning] = useState<number | null>(null)
  // Script path waiting on the "make it executable?" answer.
  const [chmodPath, setChmodPath] = useState<string | null>(null)
  const learningRef = useRef(learning)
  learningRef.current = learning

  const refresh = useCallback(async () => {
    const next = await call<State>("get_state")
    if (next) setState(next)
  }, [])

  useEffect(() => {
    void refresh()
    // Keep connection/daemon status current (plugging the pad in, etc).
    const id = setInterval(() => learningRef.current === null && void refresh(), 2000)
    return () => clearInterval(id)
  }, [refresh])

  if (!state) return <Toaster position="bottom-center" />

  const save = async (config: State["config"], msg = "Saved") => {
    if ((await call("save_config", { config })) !== undefined) toast.success(msg)
    await refresh()
  }

  const updateSlot = (i: number, slot: State["config"]["slots"][number]) => ({
    ...state.config,
    slots: state.config.slots.map((s, j) => (j === i ? slot : s)),
  })

  const lastScriptDir = () => {
    const used = state.config.slots.find((s) => s.script)?.script
    return used ? dirname(used) : state.home
  }

  const pickScript = async (i: number) => {
    const path = await open({
      title: `Script for key ${i + 1}`,
      defaultPath: state.config.slots[i].script ?? lastScriptDir(),
      filters: [
        { name: "Shell script", extensions: ["sh"] },
        { name: "All files", extensions: ["*"] },
      ],
    })
    if (typeof path !== "string") return
    await save(updateSlot(i, { ...state.config.slots[i], script: path }), `Key ${i + 1} → ${basename(path)}`)
    if (!(await call<boolean>("is_executable", { path }))) setChmodPath(path)
  }

  const learn = async (i: number) => {
    setLearning(i)
    const id = toast.loading("Press the key on your pad…")
    const learned = await call<Learned | null>("learn_key")
    toast.dismiss(id)
    setLearning(null)
    if (learned === undefined) return
    if (learned === null) return void toast.info("No key pressed, timed out")
    // A physical key belongs to one slot only.
    const slots = state.config.slots.map((s, j) =>
      j === i ? { ...s, code: learned.code } : s.code === learned.code ? { ...s, code: null } : s,
    )
    await save({ ...state.config, slots }, `Key ${i + 1} is ${learned.name}`)
  }

  const test = async (path: string) => {
    if ((await call("test_script", { path })) !== undefined) toast.success(`Ran ${basename(path)}`)
  }

  const clear = (i: number) => save(updateSlot(i, { code: null, script: null }), `Key ${i + 1} cleared`)

  const makeExecutable = async (path: string) => {
    if ((await call("make_executable", { path })) !== undefined) toast.success(`${basename(path)} is now executable`)
    await refresh()
  }

  const toggleDaemon = async () => {
    const running = state.daemon_running
    if ((await call(running ? "daemon_stop" : "daemon_start")) !== undefined) {
      toast.success(running ? "Daemon stopped" : "Daemon started")
    }
    await refresh()
  }

  const setAutostart = async (enabled: boolean) => {
    if ((await call("set_autostart", { enabled })) !== undefined) {
      toast.success(enabled ? "Daemon will start on login" : "Removed from login")
    }
    await refresh()
  }

  const grantAccess = async () => {
    if ((await call("grant_access")) !== undefined) toast.success("Access granted")
    await refresh()
  }

  const devices = state.devices.includes(state.config.device)
    ? state.devices
    : [state.config.device, ...state.devices]

  return (
    <div className="flex h-full flex-col">
      <header className="flex h-12 shrink-0 items-center gap-2 border-b border-foreground/10 px-4">
        <Keyboard className="size-4 shrink-0" />
        <h1 className="text-sm font-medium">easy-vial</h1>
        <Badge variant={state.daemon_running ? "default" : "secondary"} className="ml-1">
          {state.daemon_running ? "Daemon running" : "Daemon stopped"}
        </Badge>

        <div className="ml-auto flex items-center gap-3">
          <Label className="text-xs font-normal text-muted-foreground">
            <Switch size="sm" checked={state.autostart} onCheckedChange={setAutostart} />
            Start on login
          </Label>
          <Button size="sm" variant={state.daemon_running ? "outline" : "default"} onClick={toggleDaemon}>
            {state.daemon_running ? <Square /> : <Play />}
            {state.daemon_running ? "Stop daemon" : "Start daemon"}
          </Button>
        </div>
      </header>

      <main className="min-h-0 flex-1 overflow-y-auto p-4">
        <Card>
          <CardHeader>
            <CardTitle>Macro pad</CardTitle>
            <CardDescription className="flex items-center gap-1.5">
              <Power className={state.device_present && state.access_ok ? "size-3.5 text-success" : "size-3.5 text-destructive"} />
              {!state.device_present ? (
                "Not found, plug it in"
              ) : !state.access_ok ? (
                <>
                  No access to the device.
                  <Button variant="link" size="xs" className="h-auto px-0" onClick={grantAccess}>
                    Grant access
                  </Button>
                </>
              ) : (
                "Connected"
              )}
            </CardDescription>
            <CardAction>
              <NativeSelect
                size="sm"
                title="Input device to listen on"
                value={state.config.device}
                onChange={(e) => save({ ...state.config, device: e.target.value }, `Listening on ${e.target.value}`)}
                className="max-w-64"
              >
                {devices.map((n) => (
                  <NativeSelectOption key={n} value={n}>
                    {n}
                  </NativeSelectOption>
                ))}
              </NativeSelect>
            </CardAction>
          </CardHeader>

          <CardContent>
            <div className="grid grid-cols-3 gap-3">
              {state.config.slots.map((slot, i) => (
                <SlotCard
                  key={i}
                  index={i}
                  slot={slot}
                  keyName={state.key_names[i]}
                  executable={state.executable[i]}
                  learning={learning === i}
                  busy={learning !== null}
                  onLearn={() => learn(i)}
                  onPickScript={() => pickScript(i)}
                  onTest={() => slot.script && test(slot.script)}
                  onClear={() => clear(i)}
                  onFixExecutable={() => slot.script && setChmodPath(slot.script)}
                />
              ))}
            </div>
          </CardContent>
        </Card>

        <p className="mt-3 text-xs text-muted-foreground">
          Changes save automatically. Daemon log at <span className="font-mono">{state.log_path}</span>.
        </p>
      </main>

      <ChmodDialog
        path={chmodPath}
        onOpenChange={(o) => !o && setChmodPath(null)}
        onConfirm={() => chmodPath && makeExecutable(chmodPath)}
      />
      <Toaster position="bottom-center" />
    </div>
  )
}
