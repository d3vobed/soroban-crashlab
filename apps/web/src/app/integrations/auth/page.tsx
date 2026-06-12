import React from 'react';

import ExternalAuthenticationIntegration from '../../integrate-external-authentication-integration';

export const metadata = {
  title: 'External Auth Integration | SorobanCrashLab',
  description:
    'Dashboard panel for connecting external auth providers and validating Soroban auth-mode behavior.',
};

export default function ExternalAuthenticationIntegrationPage() {
  return (
    <div className="mx-auto max-w-7xl p-4 sm:p-6 lg:p-8">
      <ExternalAuthenticationIntegration />
    </div>
  );
}
