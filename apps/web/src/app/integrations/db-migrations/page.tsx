import React from 'react';
import DatabaseMigrationIntegrationTests from '../../integrate-database-migration-integration-tests';

export const metadata = {
  title: 'DB Migration Integration Tests | SorobanCrashLab',
  description:
    'End-to-end validation of bundle persistence schema migrations across CrashLab versions.',
};

export default function DbMigrationsIntegrationPage() {
  return (
    <div className="mx-auto max-w-7xl p-4 sm:p-6 lg:p-8">
      <DatabaseMigrationIntegrationTests />
    </div>
  );
}
