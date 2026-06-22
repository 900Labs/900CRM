import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

import { getSetting, getSettings, updateSetting } from './settings';

describe('settings api wrapper', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('maps getSettings defaults and parsers', async () => {
    invokeMock.mockResolvedValue({
      language: 'fr',
      currency: '',
      theme: 'invalid',
      date_format: '',
      sync_enabled: '1',
      sync_url: '',
    });

    const settings = await getSettings();

    expect(invokeMock).toHaveBeenCalledWith('get_settings');
    expect(settings).toEqual({
      language: 'fr',
      currency: 'USD',
      theme: 'system',
      dateFormat: 'MMM D, YYYY',
      syncEnabled: true,
      syncUrl: '',
    });
  });

  it('updates backend key/value serialization', async () => {
    invokeMock.mockResolvedValue({ key: 'sync_enabled', value: 'false', updated_at: '' });

    await updateSetting('syncEnabled', false);
    expect(invokeMock).toHaveBeenCalledWith('update_setting', {
      key: 'sync_enabled',
      value: 'false',
    });

    await updateSetting('dateFormat', 'YYYY-MM-DD');
    expect(invokeMock).toHaveBeenLastCalledWith('update_setting', {
      key: 'date_format',
      value: 'YYYY-MM-DD',
    });
  });

  it('parses getSetting values by key', async () => {
    invokeMock
      .mockResolvedValueOnce({ key: 'theme', value: 'dark', updated_at: '' })
      .mockResolvedValueOnce({ key: 'sync_enabled', value: 'true', updated_at: '' })
      .mockResolvedValueOnce(null);

    const theme = await getSetting('theme');
    const syncEnabled = await getSetting('syncEnabled');
    const language = await getSetting('language');

    expect(theme).toBe('dark');
    expect(syncEnabled).toBe(true);
    expect(language).toBe('en');
  });
});
