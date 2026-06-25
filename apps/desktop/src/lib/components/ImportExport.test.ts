// @vitest-environment jsdom

import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";

const { importDataMock, openDialogMock, preflightJsonMock } = vi.hoisted(
  () => ({
    importDataMock: vi.fn(),
    openDialogMock: vi.fn(),
    preflightJsonMock: vi.fn(),
  }),
);

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: openDialogMock,
  save: vi.fn(),
}));

vi.mock("$lib/api/importExport", async (importOriginal) => {
  const actual = await importOriginal<typeof import("$lib/api/importExport")>();

  return {
    ...actual,
    importData: importDataMock,
    preflightJson: preflightJsonMock,
  };
});

import ImportExport from "./ImportExport.svelte";

describe("ImportExport component", () => {
  beforeEach(() => {
    importDataMock.mockReset();
    openDialogMock.mockReset();
    preflightJsonMock.mockReset();
  });

  it("preflights JSON imports and requires confirmation after duplicate warnings", async () => {
    openDialogMock.mockResolvedValue("/tmp/contacts.json");
    preflightJsonMock.mockResolvedValue({
      entity_type: "contacts",
      total_rows: 1,
      duplicate_warning_count: 1,
      warnings: [
        {
          entity_type: "contacts",
          row_number: 2,
          match_type: "email",
          csv_value: "ada@example.com",
          existing_entity_type: "contact",
          existing_entity_id: "contact-1",
          existing_display_label: "Ada Lovelace",
          reason: "Email 'ada@example.com' matches existing contact",
        },
      ],
    });
    importDataMock.mockResolvedValue({
      import: { created: 1, skipped: 0, errors: [] },
      backup: {
        backup_dir: "/tmp/app-data/pre-import-backups/backup-1",
        database_path: "/tmp/app-data/pre-import-backups/backup-1/900crm.db",
        metadata_path: "/tmp/app-data/pre-import-backups/backup-1/metadata.json",
        metadata: {
          backup_format_version: 1,
          created_at: "2026-06-25T00:00:00Z",
          app_version: "0.1.0",
          schema_version: 1,
          device_id: "device-1",
          database_file: "900crm.db",
        },
      },
    });

    render(ImportExport, { open: true });

    await fireEvent.change(screen.getByLabelText("Format"), {
      target: { value: "json" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Choose File" }));

    await waitFor(() => {
      expect(preflightJsonMock).toHaveBeenCalledWith(
        "contacts",
        "/tmp/contacts.json",
      );
    });

    expect(importDataMock).not.toHaveBeenCalled();
    expect(screen.getByText("1 duplicate warnings")).toBeTruthy();
    expect(screen.getByText("Ada Lovelace")).toBeTruthy();

    await fireEvent.click(screen.getByRole("button", { name: "Continue" }));
    expect(importDataMock).not.toHaveBeenCalled();

    await fireEvent.click(
      screen.getByRole("button", { name: "Confirm import" }),
    );

    await waitFor(() => {
      expect(importDataMock).toHaveBeenCalledWith(
        "contacts",
        "json",
        "/tmp/contacts.json",
      );
    });

    expect(
      screen.getByText(
        "Automatic pre-import backup created at /tmp/app-data/pre-import-backups/backup-1",
      ),
    ).toBeTruthy();
  });
});
