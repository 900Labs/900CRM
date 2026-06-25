// @vitest-environment jsdom

import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";

const {
  importDataMock,
  openDialogMock,
  preflightJsonMock,
  previewJsonMock,
  restoreLocalBackupToAppDataMock,
  validateLocalBackupMock,
} = vi.hoisted(
  () => ({
    importDataMock: vi.fn(),
    openDialogMock: vi.fn(),
    preflightJsonMock: vi.fn(),
    previewJsonMock: vi.fn(),
    restoreLocalBackupToAppDataMock: vi.fn(),
    validateLocalBackupMock: vi.fn(),
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
    previewJson: previewJsonMock,
  };
});

vi.mock("$lib/api/backup", () => ({
  restoreLocalBackupToAppData: restoreLocalBackupToAppDataMock,
  validateLocalBackup: validateLocalBackupMock,
}));

import ImportExport from "./ImportExport.svelte";

const backupPath = "/tmp/app-data/pre-import-backups/backup-1";
const backupValidation = {
  backup_dir: backupPath,
  database_path: `${backupPath}/900crm.db`,
  metadata_path: `${backupPath}/metadata.json`,
  metadata: {
    backup_format_version: 1,
    created_at: "2026-06-25T00:00:00Z",
    app_version: "0.1.0",
    schema_version: 1,
    device_id: "device-1",
    database_file: "900crm.db",
  },
};
const importWithBackupResult = {
  import: { created: 1, skipped: 0, errors: [] },
  backup: backupValidation,
};

async function renderJsonImportSummary(result = importWithBackupResult) {
  openDialogMock.mockResolvedValue("/tmp/contacts.json");
  previewJsonMock.mockResolvedValue({
    total_rows: 1,
    headers: ["first_name", "email"],
    rows: [
      {
        row_number: 2,
        values: { first_name: "Ada", email: "ada@example.com" },
      },
    ],
  });
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
  importDataMock.mockResolvedValue(result);

  render(ImportExport, { open: true });

  await fireEvent.change(screen.getByLabelText("Format"), {
    target: { value: "json" },
  });
  await fireEvent.click(screen.getByRole("button", { name: "Choose File" }));
  await waitFor(() => {
    expect(previewJsonMock).toHaveBeenCalledWith(
      "contacts",
      "/tmp/contacts.json",
    );
  });
  await screen.findByText("Preview rows");
  await fireEvent.click(await screen.findByRole("button", { name: "Detect duplicates" }));
  await waitFor(() => {
    expect(preflightJsonMock).toHaveBeenCalledWith(
      "contacts",
      "/tmp/contacts.json",
    );
  });
  await fireEvent.click(await screen.findByRole("button", { name: "Continue" }));
  await fireEvent.click(screen.getByRole("button", { name: "Confirm import" }));

  await waitFor(() => {
    expect(importDataMock).toHaveBeenCalledWith(
      "contacts",
      "json",
      "/tmp/contacts.json",
    );
  });
  await screen.findByText("Created");
}

describe("ImportExport component", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    importDataMock.mockReset();
    openDialogMock.mockReset();
    preflightJsonMock.mockReset();
    previewJsonMock.mockReset();
    restoreLocalBackupToAppDataMock.mockReset();
    validateLocalBackupMock.mockReset();
  });

  it("previews JSON imports before duplicate preflight and confirmation", async () => {
    openDialogMock.mockResolvedValue("/tmp/contacts.json");
    previewJsonMock.mockResolvedValue({
      total_rows: 1,
      headers: ["first_name", "email"],
      rows: [
        {
          row_number: 2,
          values: { first_name: "Ada", email: "ada@example.com" },
        },
      ],
    });
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
    importDataMock.mockResolvedValue(importWithBackupResult);

    render(ImportExport, { open: true });

    await fireEvent.change(screen.getByLabelText("Format"), {
      target: { value: "json" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Choose File" }));

    await waitFor(() => {
      expect(previewJsonMock).toHaveBeenCalledWith(
        "contacts",
        "/tmp/contacts.json",
      );
    });
    expect(preflightJsonMock).not.toHaveBeenCalled();
    expect(importDataMock).not.toHaveBeenCalled();
    expect(await screen.findByText("Preview rows")).toBeTruthy();
    expect(screen.getByText("Ada")).toBeTruthy();
    expect(screen.getByText("ada@example.com")).toBeTruthy();

    await fireEvent.click(screen.getByRole("button", { name: "Detect duplicates" }));

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
        `Automatic pre-import backup created at ${backupPath}`,
      ),
    ).toBeTruthy();
    expect(screen.getByRole("button", { name: "Restore pre-import backup" })).toBeTruthy();
  });

  it("does not allow JSON duplicate preflight while preview is loading", async () => {
    let resolvePreview: (preview: {
      total_rows: number;
      headers: string[];
      rows: Array<{ row_number: number; values: Record<string, string> }>;
    }) => void = () => {};

    openDialogMock.mockResolvedValue("/tmp/contacts.json");
    previewJsonMock.mockImplementation(
      () =>
        new Promise((resolve) => {
          resolvePreview = resolve;
        }),
    );
    preflightJsonMock.mockResolvedValue({
      entity_type: "contacts",
      total_rows: 1,
      duplicate_warning_count: 0,
      warnings: [],
    });

    render(ImportExport, { open: true });

    await fireEvent.change(screen.getByLabelText("Format"), {
      target: { value: "json" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Choose File" }));

    await waitFor(() => {
      expect(previewJsonMock).toHaveBeenCalledWith(
        "contacts",
        "/tmp/contacts.json",
      );
    });

    const detectButton = screen.getByRole("button", { name: "Detect duplicates" }) as HTMLButtonElement;
    expect(detectButton.disabled).toBe(true);
    await fireEvent.click(detectButton);
    expect(preflightJsonMock).not.toHaveBeenCalled();
    expect(importDataMock).not.toHaveBeenCalled();

    resolvePreview({
      total_rows: 1,
      headers: ["first_name"],
      rows: [{ row_number: 2, values: { first_name: "Ada" } }],
    });

    await screen.findByText("Preview rows");
    const readyDetectButton = screen.getByRole("button", { name: "Detect duplicates" }) as HTMLButtonElement;
    expect(readyDetectButton.disabled).toBe(false);

    await fireEvent.click(readyDetectButton);
    await waitFor(() => {
      expect(preflightJsonMock).toHaveBeenCalledWith(
        "contacts",
        "/tmp/contacts.json",
      );
    });
  });

  it("keeps invalid JSON preview errors from running preflight or import", async () => {
    openDialogMock.mockResolvedValue("/tmp/contacts-invalid.json");
    previewJsonMock.mockRejectedValue(new Error("JSON import expects a top-level array"));

    render(ImportExport, { open: true });

    await fireEvent.change(screen.getByLabelText("Format"), {
      target: { value: "json" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Choose File" }));

    expect((await screen.findByRole("alert")).textContent).toContain(
      "JSON preview failed. Choose a valid JSON array of supported rows.",
    );
    expect(preflightJsonMock).not.toHaveBeenCalled();
    expect(importDataMock).not.toHaveBeenCalled();
    expect((screen.getByRole("button", { name: "Detect duplicates" }) as HTMLButtonElement).disabled).toBe(true);
  });

  it("does not show the restore control when the import summary has no backup path", async () => {
    await renderJsonImportSummary({
      import: { created: 1, skipped: 0, errors: [] },
      backup: { ...backupValidation, backup_dir: "" },
    });

    expect(screen.queryByRole("button", { name: "Restore pre-import backup" })).toBeNull();
  });

  it("does not restore the pre-import backup when confirmation is cancelled", async () => {
    vi.spyOn(window, "confirm").mockReturnValue(false);
    validateLocalBackupMock.mockResolvedValue(backupValidation);

    await renderJsonImportSummary();

    await screen.findByText(`Automatic pre-import backup created at ${backupPath}`);

    await fireEvent.click(screen.getByRole("button", { name: "Restore pre-import backup" }));

    await waitFor(() => {
      expect(validateLocalBackupMock).toHaveBeenCalledWith(backupPath);
    });
    expect(restoreLocalBackupToAppDataMock).not.toHaveBeenCalled();
    expect(screen.getByText("Pre-import backup restore cancelled")).toBeTruthy();
  });

  it("validates then restores the pre-import backup after explicit confirmation", async () => {
    vi.spyOn(window, "confirm").mockReturnValue(true);
    validateLocalBackupMock.mockResolvedValue(backupValidation);
    restoreLocalBackupToAppDataMock.mockResolvedValue({
      restored_at: "2026-06-25T01:00:00Z",
      database_path: "/app-data/900crm.db",
      metadata: backupValidation.metadata,
    });

    await renderJsonImportSummary();

    await screen.findByText(`Automatic pre-import backup created at ${backupPath}`);

    await fireEvent.click(screen.getByRole("button", { name: "Restore pre-import backup" }));

    await waitFor(() => {
      expect(validateLocalBackupMock).toHaveBeenCalledWith(backupPath);
    });
    expect(restoreLocalBackupToAppDataMock).toHaveBeenCalledWith(backupPath, true);
    expect(screen.getByText("Pre-import backup restored to /app-data/900crm.db")).toBeTruthy();
  });

  it("keeps the import summary visible when pre-import backup restore fails", async () => {
    vi.spyOn(window, "confirm").mockReturnValue(true);
    validateLocalBackupMock.mockResolvedValue(backupValidation);
    restoreLocalBackupToAppDataMock.mockRejectedValue(new Error("restore failed"));

    await renderJsonImportSummary();

    await screen.findByText(`Automatic pre-import backup created at ${backupPath}`);

    await fireEvent.click(screen.getByRole("button", { name: "Restore pre-import backup" }));

    await waitFor(() => {
      expect(screen.getByRole("alert").textContent).toContain(
        "Pre-import backup restore failed: restore failed",
      );
    });
    expect(screen.getByText(`Automatic pre-import backup created at ${backupPath}`)).toBeTruthy();
  });
});
