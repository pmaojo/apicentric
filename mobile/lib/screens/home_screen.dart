import 'package:flutter/material.dart';
import 'package:flutter_background_service/flutter_background_service.dart';
import 'package:path_provider/path_provider.dart';
import '../src/rust/api.dart' as rust_api;

class HomeScreen extends StatefulWidget {
  const HomeScreen({super.key});

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  bool _isRunning = false;
  List<rust_api.ServiceStatus> _services = [];
  final List<String> _logs = [];

  @override
  void initState() {
    super.initState();
    _checkServiceStatus();
  }

  Future<void> _checkServiceStatus() async {
      final service = FlutterBackgroundService();
      final isRunning = await service.isRunning();
      if (isRunning) {
          final services = await rust_api.getActiveServices();
          setState(() {
              _isRunning = true;
              _services = services;
          });
      } else {
          setState(() {
              _isRunning = false;
              _services = [];
          });
      }
  }

  Future<void> _refreshStatus() async {
    try {
      final services = await rust_api.getActiveServices();
      setState(() {
        _services = services;
      });
    } catch (e) {
      // ignore
    }
  }

  Future<void> _toggleSimulator() async {
    final service = FlutterBackgroundService();
    final isRunning = await service.isRunning();

    try {
      if (isRunning) {
        service.invoke("stopSimulator");
        service.invoke("stopService");
        setState(() {
          _isRunning = false;
          _services = [];
        });
      } else {
        await service.startService();
        final dir = await getApplicationDocumentsDirectory();
        final dbPath = "${dir.path}/apicentric.db";

        // Wait briefly for service to initialize
        await Future.delayed(const Duration(milliseconds: 500));

        service.invoke("startSimulator", {
           "servicesDir": dir.path,
           "dbPath": dbPath
        });
        setState(() {
          _isRunning = true;
        });
        // Wait for simulator to actually start before refreshing
        await Future.delayed(const Duration(seconds: 1));
        _refreshStatus();
      }
    } catch (e) {
      ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text("Error: $e")));
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text("Active Simulations")),
      body: Column(
        children: [
          Padding(
            padding: const EdgeInsets.all(16.0),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text("Status: ${_isRunning ? 'Running' : 'Stopped'}", style: Theme.of(context).textTheme.titleMedium),
                ElevatedButton(
                  onPressed: _toggleSimulator,
                  child: Text(_isRunning ? "Stop" : "Start All"),
                ),
              ],
            ),
          ),
          const Divider(),
          Expanded(
            flex: 1,
            child: _services.isEmpty
                ? const Center(child: Text("No active services"))
                : ListView.builder(
                    itemCount: _services.length,
                    itemBuilder: (context, index) {
                      final s = _services[index];
                      return ListTile(
                        leading: const Icon(Icons.api),
                        title: Text(s.name),
                        subtitle: Text("Port: ${s.port}"),
                        trailing: s.isActive ? const Icon(Icons.check_circle, color: Colors.green) : const Icon(Icons.error, color: Colors.red),
                      );
                    },
                  ),
          ),
          const Divider(),
          Padding(
            padding: const EdgeInsets.all(8.0),
            child: Row(
               mainAxisAlignment: MainAxisAlignment.spaceBetween,
               children: [
                   const Text("Logs"),
                   IconButton(icon: const Icon(Icons.refresh), onPressed: _refreshStatus)
               ]
            ),
          ),
          Expanded(
            flex: 1,
            child: Container(
              color: Colors.black12,
              child: ListView.builder(
                itemCount: _logs.length,
                itemBuilder: (context, index) {
                   return Text(_logs[index], style: const TextStyle(fontFamily: 'monospace', fontSize: 12));
                },
              ),
            ),
          ),
        ],
      ),
    );
  }
}
