import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog"
import { basename } from "@/lib/api"

/** Offers `chmod +x` for a script that isn't executable (otherwise it runs with bash). */
export function ChmodDialog({
  path,
  onOpenChange,
  onConfirm,
}: {
  path: string | null
  onOpenChange: (o: boolean) => void
  onConfirm: () => void
}) {
  return (
    <AlertDialog open={!!path} onOpenChange={onOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Not executable</AlertDialogTitle>
          <AlertDialogDescription>
            {path && basename(path)} is not executable. Make it executable (chmod +x)? Otherwise it will
            be run with bash.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Leave it</AlertDialogCancel>
          <AlertDialogAction onClick={onConfirm}>chmod +x</AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
