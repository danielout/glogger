import { useToastStore } from "../stores/toastStore";

export function useToast() {
  const store = useToastStore();

  return {
    success: (message: string) => store.add("success", message),
    info: (message: string) => store.add("info", message),
    warn: (message: string) => store.add("warning", message),
    error: (message: string) => store.add("error", message),
  };
}
