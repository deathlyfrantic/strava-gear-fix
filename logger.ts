import { Logger } from "https://deno.land/std@0.167.0/log/mod.ts";
import { ConsoleHandler } from "https://deno.land/std@0.167.0/log/handlers.ts";
import { getLevelName } from "https://deno.land/std@0.167.0/log/levels.ts";

const level = Deno.env.get("DEBUG") ? "DEBUG" : "INFO";

const handler = new ConsoleHandler(level, {
  formatter: (logRecord) =>
    `[${getLevelName(logRecord.level)}] ${
      typeof logRecord.msg === "string"
        ? logRecord.msg
        : Deno.inspect(logRecord.msg)
    } ${logRecord.args.map((arg) => " " + Deno.inspect(arg)).join(" ")}`,
});

const logger = new Logger("strava-gear-fix", level, { handlers: [handler] });

export default logger;
