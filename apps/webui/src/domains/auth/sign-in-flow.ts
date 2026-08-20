import {
  createSignal,
  readErrorPresentationDescriptor,
  readonlySignal,
} from "@securitydept/client";

export interface SignInFlowState {
  readonly pending: boolean;
  /** What to tell the user about the last attempt, if it failed. */
  readonly error: string | null;
}

const IDLE: SignInFlowState = { pending: false, error: null };

/**
 * One deliberate sign-in at a time, started from a public page. The login
 * itself belongs to the application and always runs to completion; what
 * follows it (returning to the product, reporting a failure) only happens
 * while a view is still connected to the flow.
 */
export class SignInFlow {
  private readonly options: {
    lifetimeSignal: AbortSignal;
    login(returnPath: string): Promise<void>;
  };
  private readonly current = createSignal<SignInFlowState>(IDLE);
  readonly state = readonlySignal(this.current);
  private connections = 0;
  private running = false;

  constructor(options: SignInFlow["options"]) {
    this.options = options;
  }

  connect(): void {
    this.connections++;
    // A new visit does not inherit the outcome of an earlier one.
    if (this.connections === 1 && !this.running) this.current.set(IDLE);
  }

  disconnect(): void {
    if (this.connections > 0) this.connections--;
  }

  /** Ignored while an attempt is running or nothing is connected. */
  start(returnPath: string): void {
    if (!this.observed() || this.running) return;
    this.running = true;
    this.current.set({ pending: true, error: null });
    void this.attempt(returnPath);
  }

  private observed(): boolean {
    return this.connections > 0 && !this.options.lifetimeSignal.aborted;
  }

  private async attempt(returnPath: string): Promise<void> {
    try {
      await this.options.login(returnPath);
      if (this.observed()) this.current.set(IDLE);
    } catch (reason) {
      if (this.observed())
        this.current.set({
          pending: false,
          error: readErrorPresentationDescriptor(reason).description,
        });
    } finally {
      this.running = false;
    }
  }
}
