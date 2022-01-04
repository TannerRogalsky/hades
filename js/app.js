import React, { useState } from 'react';
import { FileUploader } from 'react-drag-drop-files';

import { JsonEditor } from 'jsoneditor-react';
import 'jsoneditor-react/es/editor.min.css';

import './app.css';
import { HadesSave } from '../pkg';

const fileTypes = ['SAV'];

function DragDrop() {
	const [file, setFile] = useState(null);
	const handleNewFile = async (file) => {
		const buffer = await file.arrayBuffer();
		const bytes = new Uint8Array(buffer);
		const save = new HadesSave(bytes);
		const state = JSON.parse(save.read_json());
		setFile({
			file,
			save,
			state
		});
	};

	const handleJSONChange = (evt) => {
		file.state.GameState.Resources = evt;
	}

	const saveFile = () => {
		file.save.write_json(JSON.stringify(file.state));
		const bytes = file.save.to_bytes();
		const blob = new Blob([bytes]);
		const objectUrl = URL.createObjectURL(blob);
		const link = document.createElement('a');
		link.href = objectUrl;
		link.download = file.file.name;
		link.click();
	}

	if (file) {
		return <div>
			<JsonEditor
				value={file.state.GameState.Resources}
				onChange={handleJSONChange}
			/>
			<button onClick={saveFile}>Save File</button>
		</div>;
	} else {
		return (
			<FileUploader handleChange={handleNewFile} name='file' types={fileTypes} />
		);
	}
}

export default function App() {
	return <>
		<h1>Hades Save Editor</h1>
		<DragDrop />
	</>;
}