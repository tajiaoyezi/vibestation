import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@solidjs/testing-library";
import { ConflictBanner } from "../../src/components/ConflictBanner";

describe("ConflictBanner rollback operation", () => {
  it("renders Reverting copy with active status role", () => {
    render(() => (
      <ConflictBanner
        variant="active"
        operation="rollback"
        source="session-42"
        target={null}
        currentStep={1}
        totalSteps={3}
        filePath="src/main.rs"
        allResolved={false}
        allowSkip={false}
        onContinue={vi.fn()}
        onAbort={vi.fn()}
      />
    ));

    expect(screen.getByRole("status")).toHaveTextContent("Reverting");
  });
});
