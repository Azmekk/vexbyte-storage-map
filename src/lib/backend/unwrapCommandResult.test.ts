import { describe, expect, it } from "vitest";
import { BackendCommandError } from "./BackendCommandError";
import { unwrapCommandResult } from "./unwrapCommandResult";

type ExampleCommandError = { kind: "notFound" };

const describeExampleCommandError = () => "The item was not found.";

describe("unwrapCommandResult", () => {
  it("returns the data of a successful result", () => {
    const data = unwrapCommandResult<number, ExampleCommandError>(
      { status: "ok", data: 42 },
      describeExampleCommandError,
    );

    expect(data).toBe(42);
  });

  it("throws a BackendCommandError carrying the typed error", () => {
    const commandError: ExampleCommandError = { kind: "notFound" };

    const unwrapFailedResult = () =>
      unwrapCommandResult({ status: "error", error: commandError }, describeExampleCommandError);

    expect(unwrapFailedResult).toThrow(BackendCommandError);
    expect(unwrapFailedResult).toThrow("The item was not found.");
  });
});
