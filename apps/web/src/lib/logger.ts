type LogLevel = 'info' | 'warn' | 'error';

interface LogFields {
  [key: string]: unknown;
}

type Writer = (line: string) => void;

function defaultWriter(line: string): void {
  process.stdout.write(line + '\n');
}

function serializeFields(fields: LogFields): Record<string, unknown> {
  const out: Record<string, unknown> = {};
  for (const [k, v] of Object.entries(fields)) {
    out[k] = v instanceof Error ? v.message : v;
  }
  return out;
}

export function createLogger(writer: Writer = defaultWriter) {
  function log(level: LogLevel, msg: string, fields: LogFields = {}): void {
    const entry = {
      level,
      time: new Date().toISOString(),
      msg,
      ...serializeFields(fields),
    };
    writer(JSON.stringify(entry));
  }

  return {
    info(msg: string, fields?: LogFields): void { log('info', msg, fields ?? {}); },
    warn(msg: string, fields?: LogFields): void { log('warn', msg, fields ?? {}); },
    error(msg: string, fields?: LogFields): void { log('error', msg, fields ?? {}); },
  };
}

export const logger = createLogger();
