import { beforeEach, describe, expect, it } from "vitest";
import { selectCurrentHistory, useExplorerSelectionStore } from "../useExplorerSelectionStore";
import { canGoBack, canGoForward } from "./navigationHistory";

function explorer() {
  return useExplorerSelectionStore.getState();
}

describe("explorer folder history", () => {
  beforeEach(() => {
    useExplorerSelectionStore.setState({
      scanId: null,
      rootNode: null,
      navigationHistories: new Map(),
    });
  });

  it("records every folder change and walks back and forward", () => {
    explorer().showScan(1, 0);
    explorer().navigateToNode(5);
    explorer().navigateToNode(7, { selectedNode: 9 });
    expect(explorer().currentFolder).toBe(7);
    expect(explorer().selectedNode).toBe(9);

    explorer().goBack();
    expect(explorer().currentFolder).toBe(5);
    explorer().goBack();
    expect(explorer().currentFolder).toBe(0);
    explorer().goForward();
    expect(explorer().currentFolder).toBe(5);

    const history = selectCurrentHistory(explorer());
    expect(history !== null && canGoBack(history) && canGoForward(history)).toBe(true);
  });

  it("keeps a separate history per scan", () => {
    explorer().showScan(1, 0);
    explorer().navigateToNode(5);
    explorer().showScan(2, 100);
    expect(explorer().currentFolder).toBe(100);
    explorer().navigateToNode(101);

    explorer().showScan(1, 0);
    expect(explorer().currentFolder).toBe(5);
    explorer().goBack();
    expect(explorer().currentFolder).toBe(0);

    explorer().showScan(2, 100);
    expect(explorer().currentFolder).toBe(101);
  });

  it("selects an item in the shown folder without adding a step", () => {
    explorer().showScan(1, 0);
    explorer().navigateToNode(0, { selectedNode: 3 });
    expect(explorer().selectedNode).toBe(3);
    expect(selectCurrentHistory(explorer())?.entries).toEqual([0]);
  });

  it("replaces the current entry when the shown folder is gone", () => {
    explorer().showScan(1, 0);
    explorer().navigateToNode(5);
    explorer().navigateToNode(0, { replaceHistory: true });
    expect(selectCurrentHistory(explorer())?.entries).toEqual([0]);
  });
});
