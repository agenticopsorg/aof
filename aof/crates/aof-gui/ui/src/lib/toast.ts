// Toast notification utilities using sonner
import { toast as sonnerToast } from 'sonner';

export const toast = {
  success: (message: string, description?: string) => {
    sonnerToast.success(message, {
      description,
      duration: 3000,
    });
  },

  error: (message: string, description?: string) => {
    sonnerToast.error(message, {
      description,
      duration: 5000,
      action: description ? {
        label: 'Copy',
        onClick: () => {
          navigator.clipboard.writeText(description);
          sonnerToast.success('Error copied to clipboard');
        },
      } : undefined,
    });
  },

  warning: (message: string, description?: string) => {
    sonnerToast.warning(message, {
      description,
      duration: 4000,
    });
  },

  info: (message: string, description?: string) => {
    sonnerToast.info(message, {
      description,
      duration: 3000,
    });
  },

  loading: (message: string) => {
    return sonnerToast.loading(message);
  },

  promise: <T,>(
    promise: Promise<T>,
    messages: {
      loading: string;
      success: string | ((data: T) => string);
      error: string | ((error: Error) => string);
    }
  ) => {
    return sonnerToast.promise(promise, messages);
  },
};

// Helper to wrap Tauri invoke calls with automatic toast notifications
export async function invokeWithToast<T>(
  command: string,
  args?: Record<string, unknown>,
  options?: {
    loading?: string;
    success?: string | ((data: T) => string);
    error?: string | ((error: Error) => string);
    silent?: boolean;
  }
): Promise<T> {
  const { invoke } = await import('@tauri-apps/api/core');

  if (options?.silent) {
    return invoke<T>(command, args);
  }

  const loadingMessage = options?.loading || 'Processing...';
  const toastId = toast.loading(loadingMessage);

  try {
    const result = await invoke<T>(command, args);

    sonnerToast.dismiss(toastId);

    if (options?.success) {
      const message = typeof options.success === 'function'
        ? options.success(result)
        : options.success;
      toast.success(message);
    }

    return result;
  } catch (error) {
    sonnerToast.dismiss(toastId);

    const errorMessage = options?.error
      ? (typeof options.error === 'function'
          ? options.error(error as Error)
          : options.error)
      : 'Operation failed';

    toast.error(errorMessage, String(error));
    throw error;
  }
}
