import { readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { describe, expect, it } from 'vitest';

const srcDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const i18nDir = path.join(srcDir, 'lib', 'i18n');
const settingsSourcePath = path.join(srcDir, 'routes', 'Settings.svelte');

const warningCopy = {
  exportUnencryptedWarning:
    'CSV exports are unencrypted local files. Store files containing sensitive data in a trusted or encrypted location.',
  backupUnencryptedWarning:
    'Local backup folders are unencrypted local files. Store folders containing sensitive data in a trusted or encrypted location.',
};

function readLocaleSettings(localeFile: string): Record<string, string> {
  const source = readFileSync(path.join(i18nDir, localeFile), 'utf8');
  return JSON.parse(source).settings;
}

describe('Settings unencrypted file warnings', () => {
  it('keeps export and backup warning copy available in every locale', () => {
    const localeFiles = readdirSync(i18nDir)
      .filter((file) => file.endsWith('.json'))
      .sort();

    expect(localeFiles).toContain('en.json');

    for (const localeFile of localeFiles) {
      const settings = readLocaleSettings(localeFile);

      expect(settings.exportUnencryptedWarning).toBe(warningCopy.exportUnencryptedWarning);
      expect(settings.backupUnencryptedWarning).toBe(warningCopy.backupUnencryptedWarning);
    }
  });

  it('places warnings before the Settings export and backup actions', () => {
    const source = readFileSync(settingsSourcePath, 'utf8');

    const exportWarningIndex = source.indexOf("t('settings.exportUnencryptedWarning')");
    const exportActionIndex = source.indexOf('onclick={handleExportAll}');
    expect(exportWarningIndex).toBeGreaterThan(-1);
    expect(exportActionIndex).toBeGreaterThan(exportWarningIndex);

    const backupWarningIndex = source.indexOf("t('settings.backupUnencryptedWarning')");
    const backupActionIndex = source.indexOf('onclick={handleCreateBackup}');
    expect(backupWarningIndex).toBeGreaterThan(-1);
    expect(backupActionIndex).toBeGreaterThan(backupWarningIndex);
  });
});
