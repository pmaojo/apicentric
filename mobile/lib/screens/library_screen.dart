import 'package:flutter/material.dart';
import 'package:path_provider/path_provider.dart';
import 'dart:io';

class LibraryScreen extends StatefulWidget {
  const LibraryScreen({super.key});

  @override
  State<LibraryScreen> createState() => _LibraryScreenState();
}

class _LibraryScreenState extends State<LibraryScreen> {
  List<FileSystemEntity> _files = [];

  @override
  void initState() {
    super.initState();
    _loadFiles();
  }

  Future<void> _loadFiles() async {
    final dir = await getApplicationDocumentsDirectory();
    if (!dir.existsSync()) return;

    final List<FileSystemEntity> files = dir.listSync().where((f) => f.path.endsWith('.yaml')).toList();
    setState(() {
      _files = files;
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text("Simulations Library")),
      body: _files.isEmpty
          ? const Center(child: Text("No simulation files found."))
          : ListView.builder(
              itemCount: _files.length,
              itemBuilder: (context, index) {
                final file = _files[index];
                final name = file.path.split('/').last;
                return ListTile(
                  leading: const Icon(Icons.description),
                  title: Text(name),
                  trailing: IconButton(
                    icon: const Icon(Icons.delete),
                    onPressed: () async {
                      await file.delete();
                      _loadFiles();
                    },
                  ),
                  onTap: () {
                     // TODO: Open viewer/editor
                  },
                );
              },
            ),
      floatingActionButton: FloatingActionButton(
        onPressed: _loadFiles,
        child: const Icon(Icons.refresh),
      ),
    );
  }
}
