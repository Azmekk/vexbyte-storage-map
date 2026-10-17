import { describe, expect, it } from "vitest";
import { ScanQueryCache } from "./scanQueryCache";

function flushPromises(): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, 0));
}

describe("ScanQueryCache", () => {
  it("loads once and serves the cached result", async () => {
    const cache = new ScanQueryCache();
    let loadCount = 0;
    const load = () => {
      loadCount += 1;
      return Promise.resolve(42);
    };

    cache.ensureLoaded("1:answer", 1, load);
    cache.ensureLoaded("1:answer", 1, load);
    await flushPromises();
    cache.ensureLoaded("1:answer", 1, load);

    expect(loadCount).toBe(1);
    expect(cache.read("1:answer")).toEqual({ status: "success", data: 42, isStale: false });
  });

  it("keeps stale data visible after invalidation and refetches it", async () => {
    const cache = new ScanQueryCache();
    let nextValue = 1;
    const load = () => Promise.resolve(nextValue);

    cache.ensureLoaded("7:size", 7, load);
    await flushPromises();
    cache.invalidateScan(7);

    expect(cache.read("7:size")).toEqual({ status: "success", data: 1, isStale: true });

    nextValue = 2;
    cache.ensureLoaded("7:size", 7, load);
    await flushPromises();

    expect(cache.read("7:size")).toEqual({ status: "success", data: 2, isStale: false });
  });

  it("only invalidates the given scan", async () => {
    const cache = new ScanQueryCache();
    cache.ensureLoaded("1:a", 1, () => Promise.resolve("first"));
    cache.ensureLoaded("2:a", 2, () => Promise.resolve("second"));
    await flushPromises();

    cache.invalidateScan(1);

    expect(cache.read("2:a")).toEqual({ status: "success", data: "second", isStale: false });
  });

  it("ignores results of fetches that were superseded by an invalidation", async () => {
    const cache = new ScanQueryCache();
    let resolveFirstLoad: (value: string) => void = () => undefined;
    cache.ensureLoaded("3:slow", 3, () => Promise.resolve("before"));
    await flushPromises();
    cache.invalidateScan(3);
    cache.ensureLoaded(
      "3:slow",
      3,
      () =>
        new Promise<string>((resolve) => {
          resolveFirstLoad = resolve;
        }),
    );
    cache.removeScan(3);
    resolveFirstLoad("late");
    await flushPromises();

    expect(cache.read("3:slow")).toBeUndefined();
  });

  it("stores errors and lets them be retried", async () => {
    const cache = new ScanQueryCache();
    cache.ensureLoaded("4:broken", 4, () => Promise.reject(new Error("No access")));
    await flushPromises();

    expect(cache.read("4:broken")).toEqual({ status: "error", errorMessage: "No access" });

    cache.retry("4:broken");
    expect(cache.read("4:broken")).toBeUndefined();
  });

  it("notifies subscribers until they unsubscribe", async () => {
    const cache = new ScanQueryCache();
    let notificationCount = 0;
    const unsubscribe = cache.subscribe(() => {
      notificationCount += 1;
    });

    cache.ensureLoaded("5:x", 5, () => Promise.resolve(true));
    await flushPromises();
    unsubscribe();
    cache.invalidateScan(5);

    expect(notificationCount).toBe(1);
  });
});
