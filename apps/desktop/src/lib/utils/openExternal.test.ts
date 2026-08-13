import { beforeEach, describe, expect, it, vi } from 'vitest';

const { openMock } = vi.hoisted(() => ({
  openMock: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-shell', () => ({
  open: openMock,
}));

import { openExternalUrl, openLocalPath } from './openExternal';

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

describe('openLocalPath', () => {
  beforeEach(() => {
    openMock.mockReset();
    openMock.mockResolvedValue(undefined);
  });

  it('opens a local filesystem path', async () => {
    await openLocalPath('/Users/shared/quote.pdf');
    expect(openMock).toHaveBeenCalledWith('/Users/shared/quote.pdf');
  });

  it('rejects URL schemes', async () => {
    await expect(openLocalPath('https://evil.example/file')).rejects.toThrow(/local path/i);
    await expect(openLocalPath('file:///etc/passwd')).rejects.toThrow(/local path/i);
    expect(openMock).not.toHaveBeenCalled();
  });
});
