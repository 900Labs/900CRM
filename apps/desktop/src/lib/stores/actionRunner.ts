/**
 * src/lib/stores/actionRunner.ts — Shared async store action orchestration.
 *
 * Centralizes async action lifecycle policy so domain stores can focus on
 * state transitions instead of repeating try/catch/toast/finally scaffolding.
 */

export interface ActionNotifier {
  success: (message: string) => void;
  error: (message: string) => void;
}

interface StoreActionOptions<T> {
  action: () => Promise<T>;
  busyFlag?: (value: boolean) => void;
  notifier?: ActionNotifier;
  successMessage?: string;
  errorMessage?: string;
  onSuccess?: (value: T) => void | Promise<void>;
  onError?: (error: unknown) => void | Promise<void>;
}

export async function runStoreAction<T>(options: StoreActionOptions<T>): Promise<T> {
  options.busyFlag?.(true);

  try {
    const result = await options.action();
    await options.onSuccess?.(result);
    if (options.successMessage) {
      options.notifier?.success(options.successMessage);
    }
    return result;
  } catch (error) {
    await options.onError?.(error);
    if (options.errorMessage) {
      options.notifier?.error(options.errorMessage);
    }
    throw error;
  } finally {
    options.busyFlag?.(false);
  }
}

type LoadingActionOptions<T> = Omit<StoreActionOptions<T>, 'busyFlag'> & {
  setLoading: (value: boolean) => void;
};

type SavingActionOptions<T> = Omit<StoreActionOptions<T>, 'busyFlag'> & {
  setSaving: (value: boolean) => void;
};

export function runLoadingAction<T>(options: LoadingActionOptions<T>): Promise<T> {
  return runStoreAction({
    ...options,
    busyFlag: options.setLoading,
  });
}

export function runSavingAction<T>(options: SavingActionOptions<T>): Promise<T> {
  return runStoreAction({
    ...options,
    busyFlag: options.setSaving,
  });
}
