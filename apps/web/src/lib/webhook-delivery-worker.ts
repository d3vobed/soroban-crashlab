export type WebhookDeliveryStatus = 'queued' | 'delivered' | 'failed';

export interface WebhookDeliveryRequest {
  id: string;
  url: string;
  eventType: string;
  payload: unknown;
  headers?: Record<string, string>;
  maxAttempts?: number;
  timeoutMs?: number;
}

export interface WebhookDeliveryAttempt {
  requestId: string;
  attempt: number;
  status: WebhookDeliveryStatus;
  statusCode?: number;
  error?: string;
  deliveredAt: string;
}

export interface WebhookDeliveryAdapter {
  deliver(request: WebhookDeliveryRequest): Promise<{
    ok: boolean;
    statusCode?: number;
    error?: string;
  }>;
}

export interface WebhookDeliveryWorkerOptions {
  adapter?: WebhookDeliveryAdapter;
  maxAttempts?: number;
  retryBaseMs?: number;
  timeoutMs?: number;
  now?: () => Date;
  delay?: (ms: number) => Promise<void>;
  onAttempt?: (attempt: WebhookDeliveryAttempt) => void;
}

const DEFAULT_MAX_ATTEMPTS = 3;
const DEFAULT_RETRY_BASE_MS = 250;
const DEFAULT_TIMEOUT_MS = 5000;

export class FetchWebhookDeliveryAdapter implements WebhookDeliveryAdapter {
  async deliver(request: WebhookDeliveryRequest): Promise<{
    ok: boolean;
    statusCode?: number;
    error?: string;
  }> {
    const controller = new AbortController();
    const timeoutId = setTimeout(
      () => controller.abort(),
      request.timeoutMs ?? DEFAULT_TIMEOUT_MS,
    );

    try {
      const response = await fetch(request.url, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'X-Webhook-Event': request.eventType,
          ...request.headers,
        },
        body: JSON.stringify(request.payload),
        signal: controller.signal,
      });

      return {
        ok: response.ok,
        statusCode: response.status,
        error: response.ok ? undefined : `HTTP ${response.status}`,
      };
    } catch (error) {
      return {
        ok: false,
        error: error instanceof Error ? error.message : String(error),
      };
    } finally {
      clearTimeout(timeoutId);
    }
  }
}

export class WebhookDeliveryWorker {
  private readonly adapter: WebhookDeliveryAdapter;
  private readonly maxAttempts: number;
  private readonly retryBaseMs: number;
  private readonly timeoutMs: number;
  private readonly now: () => Date;
  private readonly delay: (ms: number) => Promise<void>;
  private readonly onAttempt?: (attempt: WebhookDeliveryAttempt) => void;
  private readonly queue: WebhookDeliveryRequest[] = [];
  private active = false;
  private draining: Promise<void> | null = null;

  constructor(options: WebhookDeliveryWorkerOptions = {}) {
    this.adapter = options.adapter ?? new FetchWebhookDeliveryAdapter();
    this.maxAttempts = options.maxAttempts ?? DEFAULT_MAX_ATTEMPTS;
    this.retryBaseMs = options.retryBaseMs ?? DEFAULT_RETRY_BASE_MS;
    this.timeoutMs = options.timeoutMs ?? DEFAULT_TIMEOUT_MS;
    this.now = options.now ?? (() => new Date());
    this.delay =
      options.delay ??
      ((ms) => new Promise((resolve) => setTimeout(resolve, ms)));
    this.onAttempt = options.onAttempt;
  }

  enqueue(request: WebhookDeliveryRequest): void {
    this.assertRequest(request);
    this.queue.push({
      ...request,
      timeoutMs: request.timeoutMs ?? this.timeoutMs,
    });

    if (this.active) {
      void this.drain();
    }
  }

  start(): void {
    if (this.active) {
      return;
    }

    this.active = true;
    void this.drain();
  }

  stop(): void {
    this.active = false;
  }

  size(): number {
    return this.queue.length;
  }

  async drain(): Promise<void> {
    if (this.draining) {
      return this.draining;
    }

    this.draining = this.processQueue();

    try {
      await this.draining;
    } finally {
      this.draining = null;
    }
  }

  private async processQueue(): Promise<void> {
    while (this.active && this.queue.length > 0) {
      const request = this.queue.shift()!;
      await this.deliverWithRetries(request);
    }
  }

  private async deliverWithRetries(
    request: WebhookDeliveryRequest,
  ): Promise<void> {
    const maxAttempts = request.maxAttempts ?? this.maxAttempts;

    for (let attempt = 1; attempt <= maxAttempts; attempt += 1) {
      const result = await this.adapter.deliver(request);
      const delivered = result.ok;
      const finalAttempt = attempt === maxAttempts;
      const willRetry =
        !delivered && !finalAttempt && this.shouldRetry(result.statusCode);

      this.recordAttempt({
        requestId: request.id,
        attempt,
        status: delivered ? 'delivered' : willRetry ? 'queued' : 'failed',
        statusCode: result.statusCode,
        error: result.error,
        deliveredAt: this.now().toISOString(),
      });

      if (!willRetry) {
        return;
      }

      await this.delay(this.retryDelayMs(attempt));
    }
  }

  private shouldRetry(statusCode?: number): boolean {
    if (statusCode === undefined) {
      return true;
    }

    if (statusCode === 429) {
      return true;
    }

    return statusCode >= 500;
  }

  private retryDelayMs(attempt: number): number {
    return this.retryBaseMs * 2 ** (attempt - 1);
  }

  private recordAttempt(attempt: WebhookDeliveryAttempt): void {
    this.onAttempt?.(attempt);
  }

  private assertRequest(request: WebhookDeliveryRequest): void {
    if (!request.id.trim()) {
      throw new Error('Webhook delivery request requires an id');
    }

    try {
      const url = new URL(request.url);
      if (url.protocol !== 'http:' && url.protocol !== 'https:') {
        throw new Error('unsupported protocol');
      }
    } catch {
      throw new Error(`Invalid webhook URL: ${request.url}`);
    }

    if (!request.eventType.trim()) {
      throw new Error('Webhook delivery request requires an event type');
    }
  }
}

export function createWebhookDeliveryWorker(
  options?: WebhookDeliveryWorkerOptions,
): WebhookDeliveryWorker {
  return new WebhookDeliveryWorker(options);
}
