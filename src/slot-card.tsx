import { TriangleAlert } from "lucide-react"

import { Button } from "@/components/ui/button"
import { Card, CardContent } from "@/components/ui/card"
import { basename, type Slot } from "@/lib/api"
import { cn } from "@/lib/utils"

/** One grid position: learned key + script, with its actions. */
export function SlotCard({
  index,
  slot,
  keyName,
  executable,
  learning,
  busy,
  onLearn,
  onPickScript,
  onTest,
  onClear,
  onFixExecutable,
}: {
  index: number
  slot: Slot
  keyName: string | null
  executable: boolean
  learning: boolean
  busy: boolean
  onLearn: () => void
  onPickScript: () => void
  onTest: () => void
  onClear: () => void
  onFixExecutable: () => void
}) {
  const ready = slot.code !== null && !!slot.script

  return (
    <Card
      size="sm"
      className={cn(
        "bg-background",
        ready && "ring-primary/40",
        learning && "ring-2 ring-primary animate-pulse",
      )}
    >
      <CardContent className="flex flex-col gap-2">
        <div className="flex items-center justify-between gap-2">
          <span className="font-heading text-sm">Key {index + 1}</span>
          <span className="truncate font-mono text-xs text-muted-foreground">
            {learning ? "press…" : (keyName ?? "not learned")}
          </span>
        </div>

        <div className="min-h-9">
          <div
            className={cn("truncate text-sm", !slot.script && "text-muted-foreground")}
            title={slot.script ?? ""}
          >
            {slot.script ? basename(slot.script) : "No script"}
          </div>
          {slot.script && !executable && (
            <button
              type="button"
              disabled={busy}
              onClick={onFixExecutable}
              className="flex items-center gap-1 text-xs text-warning hover:underline disabled:opacity-50"
            >
              <TriangleAlert className="size-3" />
              not executable, chmod +x
            </button>
          )}
        </div>

        <div className="grid grid-cols-4 gap-1">
          <Button size="xs" variant="secondary" disabled={busy} onClick={onLearn}>
            Learn
          </Button>
          <Button size="xs" variant="secondary" disabled={busy} onClick={onPickScript}>
            Script
          </Button>
          <Button size="xs" variant="secondary" disabled={busy || !slot.script} onClick={onTest}>
            Test
          </Button>
          <Button
            size="xs"
            variant="ghost"
            disabled={busy || (slot.code === null && !slot.script)}
            onClick={onClear}
          >
            Clear
          </Button>
        </div>
      </CardContent>
    </Card>
  )
}
