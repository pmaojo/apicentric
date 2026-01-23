import 'package:flutter/material.dart';
import 'package:flutter_background_service/flutter_background_service.dart';
import 'src/rust/api.dart'; // Import our Rust bridge (or mock)
import 'screens/home_screen.dart';
import 'screens/library_screen.dart';
import 'screens/create_screen.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  // Initialize Rust in main isolate
  await initApp();
  await initializeService();
  runApp(const MyApp());
}

Future<void> initializeService() async {
  final service = FlutterBackgroundService();

  await service.configure(
    androidConfiguration: AndroidConfiguration(
      onStart: onStart,
      autoStart: false,
      isForegroundMode: true,
      notificationChannelId: 'apicentric_simulator',
      initialNotificationTitle: 'Apicentric Simulator',
      initialNotificationContent: 'Simulator is running',
      foregroundServiceNotificationId: 888,
    ),
    iosConfiguration: IosConfiguration(
      autoStart: false,
      onForeground: onStart,
      onBackground: onIosBackground,
    ),
  );
}

@pragma('vm:entry-point')
void onStart(ServiceInstance service) async {
  // Initialize Rust in background isolate
  await initApp();

  service.on('stopService').listen((event) {
    service.stopSelf();
  });

  service.on('startSimulator').listen((event) async {
      final configPath = event?['configPath'] as String?;
      final dbPath = event?['dbPath'] as String?;
      if (configPath != null && dbPath != null) {
          try {
             await startSimulator(configPath: configPath, dbPath: dbPath);
             print("Simulator started via background service");
          } catch(e) {
             print("Error starting simulator: $e");
          }
      }
  });

  service.on('stopSimulator').listen((event) async {
      await stopSimulator();
  });
}

@pragma('vm:entry-point')
Future<bool> onIosBackground(ServiceInstance service) async {
  return true;
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Apicentric Mobile',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.deepPurple),
        useMaterial3: true,
      ),
      home: const MainLayout(),
    );
  }
}

class MainLayout extends StatefulWidget {
  const MainLayout({super.key});

  @override
  State<MainLayout> createState() => _MainLayoutState();
}

class _MainLayoutState extends State<MainLayout> {
  int _currentIndex = 0;

  final List<Widget> _screens = [
    const HomeScreen(),
    const LibraryScreen(),
    const CreateScreen(),
  ];

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: _screens[_currentIndex],
      bottomNavigationBar: NavigationBar(
        selectedIndex: _currentIndex,
        onDestinationSelected: (index) {
          setState(() {
            _currentIndex = index;
          });
        },
        destinations: const [
          NavigationDestination(
            icon: Icon(Icons.dashboard),
            label: 'Active',
          ),
          NavigationDestination(
            icon: Icon(Icons.library_books),
            label: 'Library',
          ),
          NavigationDestination(
            icon: Icon(Icons.add_circle_outline),
            label: 'Create',
          ),
        ],
      ),
    );
  }
}
