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

  it('maps organization country filters', async () => {
    invokeMock.mockResolvedValueOnce({
      id: 'view-org',
      entity_type: 'organization',
      name: 'Kenya accounts',
      filters_json: '{"country":"Kenya","search":"clinic"}',
      created_at: '2026-08-14T10:00:00Z',
      updated_at: '2026-08-14T10:00:00Z',
    });

    await expect(
      createSavedView('organization', 'Kenya accounts', { country: 'Kenya', search: 'clinic' }),
    ).resolves.toMatchObject({
      entityType: 'organization',
      filters: { country: 'Kenya', search: 'clinic' },
    });
  });

  it('maps deal search and custom-field filters', async () => {
    invokeMock.mockResolvedValueOnce({
      id: 'view-deal',
      entity_type: 'deal',
      name: 'Clinic rollouts',
      filters_json: '{"search":"clinic","custom_field_query":"solar"}',
      created_at: '2026-08-14T10:00:00Z',
      updated_at: '2026-08-14T10:00:00Z',
    });

    await expect(
      createSavedView('deal', 'Clinic rollouts', {
        search: 'clinic',
        customFieldQuery: 'solar',
      }),
    ).resolves.toMatchObject({
      entityType: 'deal',
      filters: { search: 'clinic', customFieldQuery: 'solar' },
    });
    expect(invokeMock).toHaveBeenCalledWith('create_saved_view', {
      entity_type: 'deal',
      name: 'Clinic rollouts',
      filters_json: JSON.stringify({
        search: 'clinic',
        custom_field_query: 'solar',
      }),
    });
  });

  it('maps deal attention filters', async () => {
    invokeMock.mockResolvedValueOnce({
      id: 'view-attention',
      entity_type: 'deal',
      name: 'Needs follow-up',
      filters_json: '{"attention":"needsFollowUp"}',
      created_at: '2026-08-14T10:00:00Z',
      updated_at: '2026-08-14T10:00:00Z',
    });

    await expect(
      createSavedView('deal', 'Needs follow-up', { attention: 'needsFollowUp' }),
    ).resolves.toMatchObject({
      entityType: 'deal',
      filters: { attention: 'needsFollowUp' },
    });
  });
});
