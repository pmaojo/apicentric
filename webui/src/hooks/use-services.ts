'use client';

import { useQuery } from '@tanstack/react-query';
import { fetchServices } from '@/services/api';

/**
 * @fileoverview A custom hook for fetching the list of API services.
 */

/**
 * Custom hook to fetch the list of API services using React Query.
 * It handles caching, refetching, and loading/error states automatically.
 * @returns {import('@tanstack/react-query').UseQueryResult<import('@/lib/types').ApiService[], Error>} The result object from React Query.
 */
export function useServices() {
  return useQuery({
    queryKey: ['services'],
    queryFn: fetchServices,
    staleTime: 5 * 60 * 1000, // 5 minutes
  });
}
