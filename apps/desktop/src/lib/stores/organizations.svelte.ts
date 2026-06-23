import {
  createOrganization,
  deleteOrganization,
  listOrganizations,
  updateOrganization,
} from '$lib/api/organizations';
import type {
  CreateOrganizationPayload,
  Organization,
  UpdateOrganizationPayload,
} from '$lib/api/organizations';
import { runLoadingAction, runSavingAction, runStoreAction } from './actionRunner';
import { uiStore } from './ui';

const notifier = {
  success: (message: string) => uiStore.toastSuccess(message),
  error: (message: string) => uiStore.toastError(message),
};

class OrganizationStore {
  organizations = $state<Organization[]>([]);
  selectedOrganization = $state<Organization | null>(null);
  isLoading = $state(false);
  isSaving = $state(false);

  async loadOrganizations(): Promise<void> {
    await runLoadingAction({
      setLoading: (value) => {
        this.isLoading = value;
      },
      notifier,
      errorMessage: 'Failed to load organizations',
      action: async () => {
        this.organizations = await listOrganizations();
      },
    });
  }

  async createOrganization(data: CreateOrganizationPayload): Promise<Organization> {
    return runSavingAction({
      setSaving: (value) => {
        this.isSaving = value;
      },
      notifier,
      successMessage: 'Organization created',
      errorMessage: 'Failed to create organization',
      action: () => createOrganization(data),
      onSuccess: (organization) => {
        this.organizations = [organization, ...this.organizations].sort((a, b) =>
          a.name.localeCompare(b.name)
        );
      },
    });
  }

  async updateOrganization(
    id: string,
    data: UpdateOrganizationPayload
  ): Promise<Organization> {
    return runSavingAction({
      setSaving: (value) => {
        this.isSaving = value;
      },
      notifier,
      successMessage: 'Organization updated',
      errorMessage: 'Failed to update organization',
      action: () => updateOrganization(id, data),
      onSuccess: (organization) => {
        this.organizations = this.organizations
          .map((existing) => (existing.id === id ? organization : existing))
          .sort((a, b) => a.name.localeCompare(b.name));
        if (this.selectedOrganization?.id === id) {
          this.selectedOrganization = organization;
        }
      },
    });
  }

  async deleteOrganization(id: string): Promise<void> {
    await runStoreAction({
      notifier,
      successMessage: 'Organization deleted',
      errorMessage: 'Failed to delete organization',
      action: () => deleteOrganization(id),
      onSuccess: () => {
        this.organizations = this.organizations.filter((organization) => organization.id !== id);
        if (this.selectedOrganization?.id === id) {
          this.selectedOrganization = null;
        }
      },
    });
  }

  selectOrganization(organization: Organization | null): void {
    this.selectedOrganization = organization;
  }
}

export const organizationStore = new OrganizationStore();
