import type {NextConfig} from 'next';

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
  experimental: {
    optimizePackageImports: ['lucide-react', '@radix-ui/react-icons'],
  },
  // Enable static optimization where possible
  output: 'standalone',
  // Optimize bundle size
  webpack: (config, { isServer }) => {
    if (!isServer) {
      // Optimize Monaco Editor loading
      config.resolve.alias = {
        ...config.resolve.alias,
        'monaco-editor': 'monaco-editor/esm/vs/editor/editor.api.js',
      };
    }
    return config;
  },
};

export default nextConfig;
