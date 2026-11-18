export type StartupState = {
  enabled: boolean;
  loaded: boolean;
  busy: boolean;
  error: string | null;
};

export interface StartupBackend {
  isAutostartEnabled(): Promise<boolean>;
  /** Applies the change and resolves with the state read back from the OS. */
  setAutostart(enabled: boolean): Promise<boolean>;
}

export const initialStartupState: StartupState = {
  enabled: false,
  loaded: false,
  busy: false,
  error: null,
};

/** Keeps the preference in sync with the OS, including failed writes. */
export class StartupPreference {
  #state = { ...initialStartupState };

  constructor(
    private backend: StartupBackend,
    private changed: (state: StartupState) => void,
  ) {}

  #publish(values: Partial<StartupState>) {
    this.#state = { ...this.#state, ...values };
    this.changed(this.#state);
  }

  async load() {
    if (this.#state.busy) return;
    this.#publish({ busy: true, error: null });
    try {
      this.#publish({ enabled: await this.backend.isAutostartEnabled(), loaded: true });
    } catch {
      this.#publish({ loaded: false, error: "Could not read startup settings. Try again." });
    } finally {
      this.#publish({ busy: false });
    }
  }

  async setEnabled(enabled: boolean) {
    if (this.#state.busy || !this.#state.loaded) return;
    this.#publish({ busy: true, error: null });
    try {
      this.#publish({ enabled: await this.backend.setAutostart(enabled), loaded: true });
    } catch {
      // A failed operation may have changed the OS before returning an error.
      // Require a fresh read before allowing another change.
      this.#publish({ loaded: false, error: "Could not update startup settings. Try again." });
    } finally {
      this.#publish({ busy: false });
    }
  }
}
