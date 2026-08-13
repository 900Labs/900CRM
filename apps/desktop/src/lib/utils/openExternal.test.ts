import { beforeEach, describe, expect, it, vi } from 'vitest';

const { openMock } = vi.hoisted(() => ({
  openMock: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-shell', () => ({
  open: openMock,
}));

import { openExternalUrl } from './openExternal';

describe('openExternalUrl', () => {
  beforeEach(() => {
    openMock.mockReset();
    openMock.mockResolvedValue(undefined);
  });

  it('opens http and https URLs', async () => {
    await openExternalUrl('https://northstar.example');
    expect(openMock).toHaveBeenCalledWith('https://northstar.example/');
  });

  it('rejects javascript and missing protocol URLs', async () => {
    await expect(openExternalUrl('javascript:alert(1)')).rejects.toThrow(/http or https/i);
    await expect(openExternalUrl('not-a-url')).rejects.toThrow();
    expect(openMock).not.toHaveBeenCalled();
  });
});
