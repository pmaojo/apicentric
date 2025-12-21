import type {NextConfig} from 'next';
import path from 'path';

const nextConfig: NextConfig = {
  /* config options here */
  typescript: {
    ignoreBuildErrors: true,
  },
  eslint: {
    ignoreDuringBuilds: true,
  },
  images: {
    remotePatterns: [
      {
        protocol: 'https',
        hostname: 'placehold.co',
        port: '',
        pathname: '/**',
      },
      {
        protocol: 'https',
        hostname: 'images.unsplash.com',
        port: '',
        pathname: '/**',
      },
      {
        protocol: 'https',
        hostname: 'picsum.photos',
        port: '',
        pathname: '/**',
      },
    ],
  },
  // Performance optimizations
  compiler: {
    removeConsole: process.env.NODE_ENV === 'production',
  },
  // Optimize imports
  experimental: {
    optimizePackageImports: ['lucide-react', '@radix-ui/react-icons'],
    // Ensure services and binary are included in the standalone build
    outputFileTracingIncludes: {
      '/api/simulate/[[...slug]]': [
        path.join(__dirname, '../services/**/*'),
        path.join(__dirname, 'backend/**/*'),
      ],
    },
  },
  // Enable static optimization where possible
  output: 'standalone',
  // Optimize bundle size
  webpack: (config, { isServer }) => {
    return config;
  },
};

export default nextConfig;
