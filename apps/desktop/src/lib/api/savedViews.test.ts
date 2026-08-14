import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

import { createSavedView, filtersMatch, listSavedViews } from './savedViews';

describe('saved views API', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('maps create and list payloads', async () => {
    invokeMock.mockResolvedValueOnce({
      id: 'view-1',
      entity_type: 'contact',
      name: 'New leads',
      filters_json: '{"lifecycle":"lead"}',
      created_at: '2026-08-13T10:00:00Z',
      updated_at: '2026-08-13T10:00:00Z',
    });

    await expect(
      createSavedView('contact', 'New leads', { lifecycle: 'lead', search: '  ' }),
    ).resolves.toMatchObject({
      id: 'view-1',
      name: 'New leads',
      filters: { lifecycle: 'lead' },
    });
    expect(invokeMock).toHaveBeenCalledWith('create_saved_view', {
      entity_type: 'contact',
      name: 'New leads',
      filters_json: JSON.stringify({ lifecycle: 'lead' }),
    });

    invokeMock.mockResolvedValueOnce([
      {
        id: 'view-1',
        entity_type: 'contact',
        name: 'New leads',
        filters_json: '{"lifecycle":"lead"}',
        created_at: '2026-08-13T10:00:00Z',
        updated_at: '2026-08-13T10:00:00Z',
      },
    ]);
    await expect(listSavedViews('contact')).resolves.toEqual([
      expect.objectContaining({ id: 'view-1', filters: { lifecycle: 'lead' } }),
    ]);
  });

  it('compares filter snapshots without empty values', () => {
    expect(filtersMatch({ lifecycle: 'lead', search: ' ' }, { lifecycle: 'lead' })).toBe(true);
    expect(filtersMatch({ lifecycle: 'lead' }, { lifecycle: 'customer' })).toBe(false);
  });
});
