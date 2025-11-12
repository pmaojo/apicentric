import { useQuery, useMutation } from '@tanstack/react-query';

export function useTestQuery(baseUrl: string) {
    return useQuery(['GET','/test'], () => fetch(`${baseUrl}/api/test`).then(res => res.json()));
}

