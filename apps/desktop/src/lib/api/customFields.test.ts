import { beforeEach, describe, expect, it, vi } from 'vitest';

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
}));

import {
  createCustomFieldDefinition,
  deleteCustomFieldDefinition,
  listCustomFieldDefinitions,
  listCustomFieldValues,
  setCustomFieldValue,
  updateCustomFieldDefinition,
} from './customFields';

describe('customFields api wrapper', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('lists field definitions by entity type', async () => {
    invokeMock.mockResolvedValue([
      {
        id: 'f1',
        entity_type: 'contact',
        field_name: 'Region',
        field_type: 'text',
        field_options: null,
        sort_order: 1,
        created_at: '2026-03-06T00:00:00Z',
      },
    ]);

    const definitions = await listCustomFieldDefinitions('contact');

    expect(invokeMock).toHaveBeenCalledWith('list_custom_field_defs', {
      entity_type: 'contact',
    });
    expect(definitions[0]).toEqual({
      id: 'f1',
      entityType: 'contact',
      fieldName: 'Region',
      fieldType: 'text',
      fieldOptions: null,
      sortOrder: 1,
      createdAt: '2026-03-06T00:00:00Z',
    });
  });

  it('creates field definition with defaults', async () => {
    invokeMock.mockResolvedValue({
      id: 'f1',
      entity_type: 'deal',
      field_name: 'Region',
      field_type: 'text',
      field_options: null,
      sort_order: 0,
      created_at: '2026-03-06T00:00:00Z',
    });

    const definition = await createCustomFieldDefinition({
      entityType: 'deal',
      fieldName: 'Region',
      fieldType: 'text',
    });

    expect(invokeMock).toHaveBeenCalledWith('create_custom_field_def', {
      entity_type: 'deal',
      field_name: 'Region',
      field_type: 'text',
      field_options: null,
      sort_order: 0,
    });
    expect(definition.fieldName).toBe('Region');
  });

  it('updates and deletes field definitions', async () => {
    invokeMock.mockResolvedValue({
      id: 'f1',
      entity_type: 'deal',
      field_name: 'Priority',
      field_type: 'text',
      field_options: null,
      sort_order: 3,
      created_at: '2026-03-06T00:00:00Z',
    });

    await updateCustomFieldDefinition({
      id: 'f1',
      fieldName: 'Priority',
      sortOrder: 3,
    });

    expect(invokeMock).toHaveBeenCalledWith('update_custom_field_def', {
      id: 'f1',
      field_name: 'Priority',
      field_type: undefined,
      field_options: undefined,
      sort_order: 3,
    });

    await deleteCustomFieldDefinition('f1');
    expect(invokeMock).toHaveBeenCalledWith('delete_custom_field_def', { id: 'f1' });
  });

  it('sets and lists entity custom field values', async () => {
    invokeMock
      .mockResolvedValueOnce({
        id: 'v1',
        field_def_id: 'f1',
        entity_id: 'c1',
        value: 'EMEA',
        created_at: '2026-03-06T00:00:00Z',
        updated_at: '2026-03-06T01:00:00Z',
      })
      .mockResolvedValueOnce([
        {
          value_id: 'v1',
          field_def_id: 'f1',
          field_name: 'Region',
          field_type: 'text',
          field_options: null,
          sort_order: 0,
          value: 'EMEA',
          updated_at: '2026-03-06T01:00:00Z',
        },
      ]);

    const value = await setCustomFieldValue({
      fieldDefId: 'f1',
      entityId: 'c1',
      value: 'EMEA',
    });
    expect(invokeMock).toHaveBeenCalledWith('set_custom_field_value', {
      field_def_id: 'f1',
      entity_id: 'c1',
      value: 'EMEA',
    });
    expect(value.fieldDefId).toBe('f1');

    const values = await listCustomFieldValues('contact', 'c1');
    expect(invokeMock).toHaveBeenCalledWith('list_custom_field_values', {
      entity_type: 'contact',
      entity_id: 'c1',
    });
    expect(values[0]?.fieldName).toBe('Region');
  });
});
